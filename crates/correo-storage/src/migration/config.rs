use crate::current::{
    Auth, ConnectionConfig as CurrentConnectionConfig, GlobalUiSettings, Lwt, MqttVersion,
    Protocol, Proxy, Settings, ThemeSettings, TlsSsl,
};
use crate::legacy::LegacyConfig;
use crate::{legacy::LegacyConnection, Result, StorageError};
use serde_json::Value;

use super::ui_config::{connection_ui_settings, message_list_view_config};
use super::{MigrationReport, MigrationWarning};

pub fn migrate_connections(
    connections: Vec<LegacyConnection>,
    report: &mut MigrationReport,
) -> Result<Vec<CurrentConnectionConfig>> {
    connections
        .into_iter()
        .enumerate()
        .map(|(index, connection)| migrate_connection(index, connection, report))
        .collect()
}

pub fn migrate_theme_settings(config: &LegacyConfig) -> Option<ThemeSettings> {
    let active_theme = config
        .themes_settings
        .as_ref()
        .and_then(|value| value.get("activeTheme"))
        .and_then(|value| value.get("name"))
        .and_then(Value::as_str)
        .map(|name| crate::current::Theme {
            name: Some(name.to_owned()),
        });

    active_theme.map(|active_theme| ThemeSettings {
        active_theme: Some(active_theme),
    })
}

pub fn migrate_settings(config: &LegacyConfig) -> Settings {
    let mut settings = Settings::default();
    let Some(value) = config.settings.as_ref() else {
        return settings;
    };

    settings.use_regex_for_search = bool_value_or(value, "useRegexForSearch", false);
    settings.use_ignore_case = bool_value_or(value, "useIgnoreCase", false);
    settings.search_updates = bool_value_or(value, "searchUpdates", false);
    settings.first_start = bool_value_or(value, "firstStart", settings.first_start);
    settings.saved_locale = string_value(value, "savedLocale");
    settings.current_locale = string_value(value, "currentLocale");
    settings.use_default_repo = bool_value_or(value, "useDefaultRepo", settings.use_default_repo);
    settings.install_bundled_plugins = bool_value_or(
        value,
        "installBundledPlugins",
        settings.install_bundled_plugins,
    );
    settings.bundled_plugins_url = string_value(value, "bundledPluginsUrl");
    settings.plugin_repositories = plugin_repositories(value);
    settings.keyring_identifier = string_value(value, "keyringIdentifier");
    settings.global_ui_settings = value.get("globalUISettings").and_then(global_ui_settings);
    settings.config_created_with_correo_version =
        string_value(value, "configCreatedWithCorreoVersion");
    settings
}

fn migrate_connection(
    index: usize,
    connection: LegacyConnection,
    report: &mut MigrationReport,
) -> Result<CurrentConnectionConfig> {
    warn_tls_review_needed(&connection, index, report);
    let id = require(connection.id, "connection", "id")?;
    let connection_ui_settings =
        connection_ui_settings(connection.connection_ui_settings.as_ref(), index, report);
    let publish_list_view_config = message_list_view_config(
        connection.publish_list_view_config.as_ref(),
        index,
        "publishListViewConfig",
        report,
    );
    let subscribe_list_view_config = message_list_view_config(
        connection.subscribe_list_view_config.as_ref(),
        index,
        "subscribeListViewConfig",
        report,
    );
    Ok(CurrentConnectionConfig {
        id,
        name: require(connection.name, "connection", "name")?,
        // The Java app is MQTT-only, so every migrated profile is MQTT.
        protocol: Protocol::Mqtt,
        url: require(connection.url, "connection", "url")?,
        port: connection.port.unwrap_or(1883),
        client_id: connection.client_id,
        username: connection.username,
        clean_session: connection.clean_session,
        mqtt_version: mqtt_version(connection.mqtt_version.as_deref(), index, report),
        ssl: tls_mode(connection.ssl.as_deref(), index, report),
        ssl_keystore: connection.ssl_keystore,
        ssl_host_verification: connection.ssl_host_verification,
        proxy: proxy_mode(connection.proxy.as_deref(), index, report),
        ssh_host: connection.ssh_host,
        ssh_port: connection.ssh_port.unwrap_or(22),
        local_port: connection.local_port,
        auth: auth_mode(connection.auth.as_deref(), index, report),
        auth_username: connection.auth_username,
        auth_keyfile: connection.auth_keyfile,
        lwt: lwt_mode(connection.lwt.as_deref(), index, report),
        lwt_topic: connection.lwt_topic,
        lwt_qos: connection
            .lwt_qo_s
            .and_then(crate::current::Qos::from_legacy),
        lwt_retained: connection.lwt_retained,
        lwt_payload: connection.lwt_payload,
        connection_ui_settings,
        publish_list_view_config,
        subscribe_list_view_config,
        plugin_workflows: Vec::new(),
    })
}

fn warn_tls_review_needed(
    connection: &LegacyConnection,
    index: usize,
    report: &mut MigrationReport,
) {
    let tls_enabled = matches!(
        normalized(connection.ssl.as_deref()).as_deref(),
        Some("ON" | "KEYSTORE" | "SSL" | "TLS")
    );
    if !tls_enabled {
        return;
    }

    if let Some(path) = connection.ssl_keystore.as_deref() {
        let lower = path.to_ascii_lowercase();
        if lower.ends_with(".p12") || lower.ends_with(".pfx") {
            report.warnings.push(MigrationWarning {
                code: "legacy_connection_tls_pkcs12_review",
                message: format!(
                    "Legacy connection config.connections[{index}].sslKeystore uses PKCS#12, which is not supported; convert the client certificate to PEM (openssl pkcs12 -in keystore.p12 -out cert.pem) and re-select it in the connection settings"
                ),
            });
        } else {
            report.warnings.push(MigrationWarning {
                code: "legacy_connection_tls_keystore_review",
                message: format!(
                    "Legacy connection config.connections[{index}].sslKeystore uses a Java keystore, which is not supported; export the client certificate to PEM (keytool -importkeystore to PKCS#12, then openssl pkcs12) and re-select it in the connection settings"
                ),
            });
        }
    }

    if !connection.ssl_host_verification {
        report.warnings.push(MigrationWarning {
            code: "legacy_connection_tls_host_verification_review",
            message: format!(
                "Legacy connection config.connections[{index}] disables TLS host verification; Rust currently requires manual review for this insecure option"
            ),
        });
    }

    if matches!(
        normalized(connection.proxy.as_deref()).as_deref(),
        Some("SSH")
    ) {
        report.warnings.push(MigrationWarning {
            code: "legacy_connection_tls_over_ssh_review",
            message: format!(
                "Legacy connection config.connections[{index}] combines TLS and SSH tunneling; Rust currently requires manual review for TLS-over-SSH"
            ),
        });
    }
}

fn mqtt_version(value: Option<&str>, index: usize, report: &mut MigrationReport) -> MqttVersion {
    match value {
        Some("MQTT_5_0" | "MQTT5" | "MQTT 5") => MqttVersion::Mqtt50,
        Some("MQTT_3_1_1" | "MQTT311" | "MQTT 3.1.1") | None => MqttVersion::Mqtt311,
        Some(other) => {
            report.warnings.push(MigrationWarning {
                code: "legacy_connection_mqtt_version_unknown",
                message: format!(
                    "Legacy connection config.connections[{index}].mqttVersion={other} defaulted to MQTT 3.1.1"
                ),
            });
            MqttVersion::Mqtt311
        }
    }
}

fn tls_mode(value: Option<&str>, index: usize, report: &mut MigrationReport) -> TlsSsl {
    match normalized(value).as_deref() {
        Some("OFF") | None => TlsSsl::Off,
        Some("ON" | "KEYSTORE" | "SSL" | "TLS") => TlsSsl::Keystore,
        Some(other) => {
            report.warnings.push(MigrationWarning {
                code: "legacy_connection_tls_unknown",
                message: format!(
                    "Legacy connection config.connections[{index}].ssl={other} defaulted to disabled"
                ),
            });
            TlsSsl::Off
        }
    }
}

fn proxy_mode(value: Option<&str>, index: usize, report: &mut MigrationReport) -> Proxy {
    match normalized(value).as_deref() {
        Some("OFF") | None => Proxy::Off,
        Some("SSH") => Proxy::Ssh,
        Some(other) => {
            report.warnings.push(MigrationWarning {
                code: "legacy_connection_proxy_unknown",
                message: format!(
                    "Legacy connection config.connections[{index}].proxy={other} defaulted to disabled"
                ),
            });
            Proxy::Off
        }
    }
}

fn auth_mode(value: Option<&str>, index: usize, report: &mut MigrationReport) -> Auth {
    match normalized(value).as_deref() {
        Some("OFF") | None => Auth::Off,
        Some("PASSWORD") => Auth::Password,
        Some("KEY" | "KEYFILE") => Auth::Keyfile,
        Some(other) => {
            report.warnings.push(MigrationWarning {
                code: "legacy_connection_auth_unknown",
                message: format!(
                    "Legacy connection config.connections[{index}].auth={other} defaulted to disabled"
                ),
            });
            Auth::Off
        }
    }
}

fn lwt_mode(value: Option<&str>, index: usize, report: &mut MigrationReport) -> Lwt {
    match normalized(value).as_deref() {
        Some("OFF") | None => Lwt::Off,
        Some("ON") => Lwt::On,
        Some(other) => {
            report.warnings.push(MigrationWarning {
                code: "legacy_connection_lwt_unknown",
                message: format!(
                    "Legacy connection config.connections[{index}].lwt={other} defaulted to disabled"
                ),
            });
            Lwt::Off
        }
    }
}

fn bool_value_or(value: &Value, key: &str, fallback: bool) -> bool {
    value.get(key).and_then(Value::as_bool).unwrap_or(fallback)
}

fn string_value(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

fn plugin_repositories(value: &Value) -> std::collections::BTreeMap<String, String> {
    value
        .get("pluginRepositories")
        .and_then(Value::as_object)
        .map(|repositories| {
            repositories
                .iter()
                .filter_map(|(id, url)| Some((id.clone(), url.as_str()?.to_owned())))
                .collect()
        })
        .unwrap_or_default()
}

fn global_ui_settings(value: &Value) -> Option<GlobalUiSettings> {
    Some(GlobalUiSettings {
        window_position_x: f64_field(value, "windowPositionX")?,
        window_position_y: f64_field(value, "windowPositionY")?,
        window_width: f64_field(value, "windowWidth")?,
        window_height: f64_field(value, "windowHeight")?,
    })
}

fn f64_field(value: &Value, key: &str) -> Option<f64> {
    value.get(key).and_then(Value::as_f64)
}

fn normalized(value: Option<&str>) -> Option<String> {
    value.map(|value| value.trim().to_ascii_uppercase())
}

fn require(value: Option<String>, record: &'static str, field: &'static str) -> Result<String> {
    value.ok_or(StorageError::MissingField { record, field })
}
