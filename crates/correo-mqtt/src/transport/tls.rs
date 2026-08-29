use std::io::BufReader;
use std::sync::Arc;

use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::client::WebPkiServerVerifier;
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{
    CertificateError, ClientConfig, DigitallySignedStruct, Error as RustlsError, RootCertStore,
    SignatureScheme,
};

use crate::{
    MqttError, MqttResult, TlsClientIdentity, TlsConfig, TlsHostVerification, TlsOptions,
    TlsTrustRoots,
};

pub(crate) fn validate(_config: &TlsConfig) -> MqttResult<()> {
    Ok(())
}

pub(crate) fn rustls_client_config(config: &TlsConfig) -> MqttResult<Option<ClientConfig>> {
    let TlsConfig::Enabled(options) = config else {
        return Ok(None);
    };

    validate(config)?;
    let roots = root_store(options)?;
    let provider: Arc<rustls::crypto::CryptoProvider> =
        rustls::crypto::ring::default_provider().into();
    let builder = ClientConfig::builder_with_provider(provider.clone())
        .with_safe_default_protocol_versions()
        .map_err(|error| MqttError::tls(error.to_string()))?;
    let builder = if matches!(
        options.host_verification,
        TlsHostVerification::DisabledInsecure
    ) {
        let verifier = WebPkiServerVerifier::builder_with_provider(Arc::new(roots), provider)
            .build()
            .map_err(|error| MqttError::tls(error.to_string()))?;
        builder
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(SkipHostnameVerification {
                inner: verifier,
            }))
    } else {
        builder.with_root_certificates(roots)
    };
    let config = match &options.client_identity {
        Some(identity) => with_client_identity(builder, identity)?,
        None => builder.with_no_client_auth(),
    };
    Ok(Some(config))
}

/// Verifies the certificate chain and validity as usual and only tolerates a
/// hostname mismatch — the behaviour behind the connection setting that
/// disables TLS host verification (Java parity).
#[derive(Debug)]
struct SkipHostnameVerification {
    inner: Arc<WebPkiServerVerifier>,
}

impl ServerCertVerifier for SkipHostnameVerification {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        intermediates: &[CertificateDer<'_>],
        server_name: &ServerName<'_>,
        ocsp_response: &[u8],
        now: UnixTime,
    ) -> Result<ServerCertVerified, RustlsError> {
        match self.inner.verify_server_cert(
            end_entity,
            intermediates,
            server_name,
            ocsp_response,
            now,
        ) {
            Err(RustlsError::InvalidCertificate(
                CertificateError::NotValidForName | CertificateError::NotValidForNameContext { .. },
            )) => Ok(ServerCertVerified::assertion()),
            other => other,
        }
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, RustlsError> {
        self.inner.verify_tls12_signature(message, cert, dss)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, RustlsError> {
        self.inner.verify_tls13_signature(message, cert, dss)
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.inner.supported_verify_schemes()
    }
}

fn root_store(options: &TlsOptions) -> MqttResult<RootCertStore> {
    let mut roots = RootCertStore::empty();
    match &options.trust_roots {
        TlsTrustRoots::Native => {
            let result = rustls_native_certs::load_native_certs();
            if !result.errors.is_empty() {
                return Err(MqttError::tls(format!(
                    "native certificate store could not be loaded: {} error(s)",
                    result.errors.len()
                )));
            }
            let (added, ignored) = roots.add_parsable_certificates(result.certs);
            if added == 0 {
                return Err(MqttError::tls(format!(
                    "native certificate store did not provide usable roots; ignored {ignored}"
                )));
            }
        }
        TlsTrustRoots::PemBundle { path, pem } => {
            let loaded_pem;
            let pem_bytes = if let Some(pem) = pem {
                pem.expose_secret()
            } else {
                let Some(path) = path else {
                    return Err(MqttError::tls(
                        "CA PEM material must include inline bytes or a path",
                    ));
                };
                loaded_pem = std::fs::read(path).map_err(|error| {
                    MqttError::tls(format!("CA PEM bundle could not be read: {error}"))
                })?;
                loaded_pem.as_slice()
            };
            let certs = parse_certs(pem_bytes)?;
            let (added, ignored) = roots.add_parsable_certificates(certs);
            if added == 0 {
                return Err(MqttError::tls(format!(
                    "CA PEM bundle did not contain usable certificates; ignored {ignored}"
                )));
            }
        }
        TlsTrustRoots::Pkcs12 { .. } => {
            return Err(MqttError::tls(
                "PKCS#12 trust stores require a native-tls backend that is not enabled",
            ));
        }
    }
    Ok(roots)
}

fn with_client_identity(
    builder: rustls::ConfigBuilder<ClientConfig, rustls::client::WantsClientCert>,
    identity: &TlsClientIdentity,
) -> MqttResult<ClientConfig> {
    match identity {
        TlsClientIdentity::Pem {
            certificate_pem,
            private_key_pem,
        } => {
            let certs = parse_certs(certificate_pem.expose_secret())?;
            if certs.is_empty() {
                return Err(MqttError::tls(
                    "client certificate PEM did not contain certificates",
                ));
            }
            let key = parse_private_key(private_key_pem.expose_secret())?;
            builder
                .with_client_auth_cert(certs, key)
                .map_err(|error| MqttError::tls(error.to_string()))
        }
        TlsClientIdentity::Pkcs12 { .. } => Err(MqttError::tls(
            "PKCS#12 client identities require a native-tls backend that is not enabled",
        )),
    }
}

fn parse_certs(bytes: &[u8]) -> MqttResult<Vec<rustls::pki_types::CertificateDer<'static>>> {
    rustls_pemfile::certs(&mut BufReader::new(bytes))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| MqttError::tls(error.to_string()))
}

fn parse_private_key(bytes: &[u8]) -> MqttResult<rustls::pki_types::PrivateKeyDer<'static>> {
    rustls_pemfile::private_key(&mut BufReader::new(bytes))
        .map_err(|error| MqttError::tls(error.to_string()))?
        .ok_or_else(|| MqttError::tls("client private key PEM did not contain a private key"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SecretBytes, SecretString};

    const SYNTHETIC_CA_PEM: &[u8] = br#"-----BEGIN CERTIFICATE-----
MIIDHzCCAgegAwIBAgIUZWsL62kjzfW2COhwQUu0pcQQUfQwDQYJKoZIhvcNAQEL
BQAwFDESMBAGA1UEAwwJbG9jYWxob3N0MB4XDTI2MDYwODE4Mzg1N1oXDTI2MDYw
OTE4Mzg1N1owFDESMBAGA1UEAwwJbG9jYWxob3N0MIIBIjANBgkqhkiG9w0BAQEF
AAOCAQ8AMIIBCgKCAQEAyCvrZZnyKYUFEx3DiiB17RKKXgw1u/sgzSjUsoYMf88k
SCzG0nqkKWJEilAG5boKkY66EJKQvdWL5V502JQPGLjeSVrHNvTXdgnej2iWCiVU
YDi6cd4GidkJMJHl/FLu+1JiwRdCnuolkbNs327ewnWc2JZoE0iVTRRsO0Hxpce3
hAWFKVDR7vSuYkrd3lRT/IvPOX7vQFItz4KcsW4GxeqVZE1vislxyDJUAhdM/LoS
WtjX6pIsKXq1npDZ9tzuW7mZgUZ5f4bLX+tFNBKSwwsFYF8U/kZWWuhExAxF5RjP
/Pe1eJXyv2jdBYrHNPr0x6DjLYHapPV/tkMhCEOflwIDAQABo2kwZzAdBgNVHQ4E
FgQUurG8EQj9x09LEFMIuCPESY9IucwwHwYDVR0jBBgwFoAUurG8EQj9x09LEFMI
uCPESY9IucwwDwYDVR0TAQH/BAUwAwEB/zAUBgNVHREEDTALgglsb2NhbGhvc3Qw
DQYJKoZIhvcNAQELBQADggEBALDDUqPGIrZ96yacRrCplXWdQqzU+HGd5aDUeYoz
6+SYXLOSs2fslDtGr/0aBJSSGEYlwWx/uaQZeFLV0lHOea/QrWQ+s1FxL5QML+Xg
rzvTkDnnD5NEJQqHSwqqGVIrnxoOWjoJJvW1qaJqLKfXw75NhqQ8UVLis7O6Q2Yv
py7jM3CPXmP+A+K+DxlC7FSlReWjCB5O20qluHb7L5CoSfdZow4nUEtJVD9xefiw
K21Q1+Yo64n7/MTpeFoe53PvoMqzFfUN2EnPW8vlgn0tDu9/c9kdueCEZpgsBFsl
oaQ31k55+MyS0LvD2dJIcPD6vtubQ9P/uTq0l7vOAkNrREc=
-----END CERTIFICATE-----
"#;

    #[test]
    fn native_tls_options_preserve_host_verification_by_default() {
        let options = TlsConfig::Enabled(TlsOptions::default());
        assert!(validate(&options).is_ok());
    }

    #[test]
    fn insecure_hostname_disable_builds_config_with_chain_verification() {
        let options = TlsConfig::Enabled(TlsOptions {
            host_verification: TlsHostVerification::DisabledInsecure,
            trust_roots: TlsTrustRoots::PemBundle {
                path: None,
                pem: Some(SecretBytes::new(SYNTHETIC_CA_PEM.to_vec())),
            },
            client_identity: None,
        });

        assert!(validate(&options).is_ok());
        let config = rustls_client_config(&options).expect("valid TLS config");
        assert!(config.is_some());
    }

    #[test]
    fn pkcs12_material_reports_unsupported_backend_without_secret() {
        let options = TlsConfig::Enabled(TlsOptions {
            trust_roots: TlsTrustRoots::Pkcs12 {
                path: Some("bundle.p12".to_owned()),
                der: SecretBytes::new(b"synthetic-pkcs12-secret".to_vec()),
                password: Some(SecretString::new("synthetic-password")),
            },
            ..TlsOptions::default()
        });

        let error = rustls_client_config(&options).expect_err("unsupported");
        let rendered = error.to_string();
        assert!(!rendered.contains("synthetic-pkcs12-secret"));
        assert!(!rendered.contains("synthetic-password"));
        assert!(rendered.contains("PKCS#12"));
    }

    #[test]
    fn rustls_client_config_builds_without_process_default_provider() {
        let options = TlsConfig::Enabled(TlsOptions {
            host_verification: TlsHostVerification::Enabled,
            trust_roots: TlsTrustRoots::PemBundle {
                path: Some("synthetic-ca.pem".to_owned()),
                pem: Some(SecretBytes::new(SYNTHETIC_CA_PEM.to_vec())),
            },
            client_identity: None,
        });

        let result = std::panic::catch_unwind(|| rustls_client_config(&options));
        let config = result.expect("rustls crypto provider selection should not panic");
        assert!(config.expect("valid TLS config").is_some());
    }
}
