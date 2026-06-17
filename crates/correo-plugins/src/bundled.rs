use crate::{
    CapabilityGrants, ConfigSchemaMetadata, ConnectionHeaderActionContribution, HookInvocation,
    HookKind, HookOutput, IncomingMessageTransformResponse, IntoPluginDiagnostic,
    MessageTransformOutcomeDto, MessageValidatorResponse, OutgoingMessageTransformResponse,
    PluginDiagnostic, PluginEntrypoint, PluginManifest, ValidationResultDto, ABI_VERSION,
};
use advanced_validator::{
    config_schema as advanced_validator_config_schema,
    validate_message as validate_advanced_validator_message,
};
use base64::Engine;
use correo_plugins_advanced_validator::AdvancedValidatorError;
use correo_plugins_systopic::{LEGACY_PLUGIN_ID as SYSTOPIC_LEGACY_ID, PLUGIN_ID as SYSTOPIC_ID};
use correo_plugins_xml_xsd_validator::{
    config_schema_document, validate_xml_xsd, XmlXsdValidation,
};
use formatting::{format_json, format_xml};
pub use formatting::{highlight_json, highlight_xml, PayloadSyntaxKind, PayloadSyntaxSpan};
use gzip::{gzip_config_schema, transform_gzip_detail, GzipTransformError};
use semver::{Version, VersionReq};
use serde::Deserialize;
use serde_json::{json, Value};
use thiserror::Error;

mod advanced_validator;
mod formatting;
mod gzip;

const BASE64_ID: &str = "org.correomqtt.plugins.base64";
pub const JSON_FORMAT_ID: &str = "org.correomqtt.plugins.json-format";
pub const XML_FORMAT_ID: &str = "org.correomqtt.plugins.xml-format";
const CONTAINS_STRING_ID: &str = "org.correomqtt.plugins.contains-string-validator";
const ADVANCED_VALIDATOR_ID: &str = "org.correomqtt.plugins.advanced-validator";
const XML_XSD_VALIDATOR_ID: &str = "org.correomqtt.plugins.xml-xsd-validator";
const ZIP_MANIPULATOR_ID: &str = "org.correomqtt.plugins.zip-manipulator";
pub const SAVE_MANIPULATOR_ID: &str = "org.correomqtt.plugins.save-manipulator";

#[derive(Debug, Clone)]
pub struct BundledPlugin {
    manifest: PluginManifest,
    kind: BundledPluginKind,
}

impl BundledPlugin {
    pub fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    pub fn id(&self) -> &str {
        &self.manifest.id
    }

    pub fn dispatch(&self, invocation: HookInvocation) -> Result<HookOutput, BundledPluginError> {
        let hook = invocation.hook();
        if !self.manifest.capabilities.grants_hook(hook) {
            return Err(BundledPluginError::HookNotDeclared {
                plugin_id: self.id().to_owned(),
                hook,
            });
        }

        match (self.kind, invocation) {
            (BundledPluginKind::Base64, HookInvocation::OutgoingMessageTransform(mut request)) => {
                request.message.payload = base64::engine::general_purpose::STANDARD
                    .encode(&request.message.payload)
                    .into_bytes();
                Ok(HookOutput::OutgoingMessageTransform(
                    OutgoingMessageTransformResponse {
                        abi_version: ABI_VERSION,
                        outcome: MessageTransformOutcomeDto::Replace {
                            message: request.message,
                        },
                    },
                ))
            }
            (BundledPluginKind::Base64, HookInvocation::IncomingMessageTransform(mut request)) => {
                let outcome = match base64::engine::general_purpose::STANDARD
                    .decode(&request.message.payload)
                {
                    Ok(decoded) => {
                        request.message.payload = decoded;
                        MessageTransformOutcomeDto::Replace {
                            message: request.message,
                        }
                    }
                    Err(_) => MessageTransformOutcomeDto::Unchanged,
                };
                Ok(HookOutput::IncomingMessageTransform(
                    IncomingMessageTransformResponse {
                        abi_version: ABI_VERSION,
                        outcome,
                    },
                ))
            }
            (BundledPluginKind::JsonFormatter, HookInvocation::DetailFormatter(request)) => {
                format_json(request, self.id()).map(HookOutput::DetailFormatter)
            }
            (BundledPluginKind::XmlFormatter, HookInvocation::DetailFormatter(request)) => {
                format_xml(request, self.id()).map(HookOutput::DetailFormatter)
            }
            (
                BundledPluginKind::ContainsStringValidator,
                HookInvocation::MessageValidator(request),
            ) => validate_contains_string(request.config, request.message.payload, self.id())
                .map(HookOutput::MessageValidator),
            (BundledPluginKind::AdvancedValidator, HookInvocation::MessageValidator(request)) => {
                validate_advanced_validator_message(request.config, &request.message.payload)
                    .map(HookOutput::MessageValidator)
                    .map_err(
                        |source| BundledPluginError::InvalidAdvancedValidatorConfig {
                            plugin_id: self.id().to_owned(),
                            hook: HookKind::MessageValidator,
                            source,
                        },
                    )
            }
            (BundledPluginKind::XmlXsdValidator, HookInvocation::MessageValidator(request)) => {
                Ok(HookOutput::MessageValidator(validate_xml_xsd_message(
                    request.config,
                    &request.message.payload,
                )))
            }
            (BundledPluginKind::ZipManipulator, HookInvocation::DetailByteTransform(request)) => {
                transform_gzip_detail(request, self.id()).map(HookOutput::DetailByteTransform)
            }
            (_, invocation) => Err(BundledPluginError::HookNotDeclared {
                plugin_id: self.id().to_owned(),
                hook: invocation.hook(),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BundledPluginKind {
    Base64,
    JsonFormatter,
    XmlFormatter,
    ContainsStringValidator,
    AdvancedValidator,
    XmlXsdValidator,
    ZipManipulator,
    SystemTopicFormatter,
}

pub fn bundled_plugins() -> Vec<BundledPlugin> {
    vec![
        bundled_plugin(
            BASE64_ID,
            "Base64 Transform",
            "Encodes outgoing payloads and decodes incoming payloads with Base64.",
            &[
                HookKind::OutgoingMessageTransform,
                HookKind::IncomingMessageTransform,
            ],
            empty_config_schema(),
            BundledPluginKind::Base64,
        ),
        bundled_plugin(
            JSON_FORMAT_ID,
            "JSON Formatter",
            "Formats valid JSON payloads for the message detail view.",
            &[HookKind::DetailFormatter, HookKind::PayloadHighlighter],
            empty_config_schema(),
            BundledPluginKind::JsonFormatter,
        ),
        bundled_plugin(
            XML_FORMAT_ID,
            "XML Formatter",
            "Formats XML-like payloads for the message detail view.",
            &[HookKind::DetailFormatter, HookKind::PayloadHighlighter],
            empty_config_schema(),
            BundledPluginKind::XmlFormatter,
        ),
        bundled_plugin(
            CONTAINS_STRING_ID,
            "Contains String Validator",
            "Validates text payloads by checking for configured text.",
            &[HookKind::MessageValidator],
            contains_string_config_schema(),
            BundledPluginKind::ContainsStringValidator,
        ),
        bundled_plugin(
            ADVANCED_VALIDATOR_ID,
            "Advanced Validator",
            "Composes configured validator rules with AND and OR groups.",
            &[HookKind::MessageValidator],
            advanced_validator_config_schema(),
            BundledPluginKind::AdvancedValidator,
        ),
        system_topic_plugin(),
        bundled_plugin(
            XML_XSD_VALIDATOR_ID,
            "XML/XSD Validator",
            "Validates XML payloads against configured inline XSD schema text.",
            &[HookKind::MessageValidator],
            xml_xsd_config_schema(),
            BundledPluginKind::XmlXsdValidator,
        ),
        bundled_plugin(
            ZIP_MANIPULATOR_ID,
            "ZIP Manipulator",
            "Compresses or decompresses message detail bytes with GZIP.",
            &[HookKind::DetailByteTransform],
            gzip_config_schema(),
            BundledPluginKind::ZipManipulator,
        ),
    ]
}

fn system_topic_plugin() -> BundledPlugin {
    let manifest = PluginManifest {
        manifest_version: 1,
        id: SYSTOPIC_ID.to_owned(),
        name: "System Topics".to_owned(),
        version: Version::new(0, 1, 0),
        description: "Adds a connection header action that opens live $SYS broker metrics."
            .to_owned(),
        provider: "CorreoMQTT".to_owned(),
        license: "GPL-3.0-or-later".to_owned(),
        compatible_correomqtt: VersionReq::parse(">=0.1.0, <1.0.0")
            .expect("bundled compatibility requirement is valid"),
        capabilities: CapabilityGrants {
            hooks: Vec::new(),
            host: crate::HostCapabilityGrants {
                mqtt: true,
                ui: true,
                ..Default::default()
            },
        },
        entrypoints: Vec::new(),
        connection_header_actions: vec![ConnectionHeaderActionContribution {
            id: "system-topics".to_owned(),
            label: "$SYS".to_owned(),
            tooltip: "Open system topics".to_owned(),
            requires_connected: true,
        }],
        themes: Vec::new(),
        config_schema: Some(empty_config_schema()),
    };
    manifest
        .validate()
        .expect("bundled plugin manifest is valid");
    BundledPlugin {
        manifest,
        kind: BundledPluginKind::SystemTopicFormatter,
    }
}

pub fn bundled_plugin_manifests() -> Vec<PluginManifest> {
    bundled_plugins()
        .into_iter()
        .map(|plugin| plugin.manifest)
        .collect()
}

pub fn bundled_plugin_by_id(plugin_id: &str) -> Option<BundledPlugin> {
    bundled_plugins()
        .into_iter()
        .find(|plugin| plugin.id() == plugin_id)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyPluginReplacementDecision {
    pub legacy_plugin_id: &'static str,
    pub status: LegacyPluginReplacementStatus,
    pub replacement_plugin_id: Option<&'static str>,
    pub reason: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegacyPluginReplacementStatus {
    Supported,
    Unsupported,
}

pub fn legacy_plugin_replacement_decisions() -> Vec<LegacyPluginReplacementDecision> {
    vec![
        supported_builtin("base64", BASE64_ID),
        supported_builtin("json-format", JSON_FORMAT_ID),
        supported_builtin("xml-format", XML_FORMAT_ID),
        supported_builtin("contains-string-validator", CONTAINS_STRING_ID),
        supported_builtin("advanced-validator", ADVANCED_VALIDATOR_ID),
        supported_builtin("xml-xsd-validator", XML_XSD_VALIDATOR_ID),
        supported_builtin("zip-manipulator", ZIP_MANIPULATOR_ID),
        supported(
            "save-manipulator",
            SAVE_MANIPULATOR_ID,
            "Covered by a Rust/WASM plugin that requests host-mediated payload saves.",
        ),
        supported_builtin(SYSTOPIC_LEGACY_ID, SYSTOPIC_ID),
    ]
}

#[derive(Debug, Error)]
pub enum BundledPluginError {
    #[error("bundled plugin {plugin_id} does not declare hook {hook:?}")]
    HookNotDeclared { plugin_id: String, hook: HookKind },
    #[error("bundled plugin {plugin_id} received non-UTF-8 payload for hook {hook:?}: {source}")]
    InvalidUtf8 {
        plugin_id: String,
        hook: HookKind,
        source: std::string::FromUtf8Error,
    },
    #[error("bundled plugin {plugin_id} received invalid config for hook {hook:?}: {source}")]
    InvalidConfig {
        plugin_id: String,
        hook: HookKind,
        source: serde_json::Error,
    },
    #[error("bundled plugin {plugin_id} received invalid advanced validator config for {hook:?}: {source}")]
    InvalidAdvancedValidatorConfig {
        plugin_id: String,
        hook: HookKind,
        source: AdvancedValidatorError,
    },
    #[error("bundled plugin {plugin_id} failed {hook:?}: {source}")]
    TransformFailed {
        plugin_id: String,
        hook: HookKind,
        source: GzipTransformError,
    },
}

impl IntoPluginDiagnostic for BundledPluginError {
    fn diagnostic(&self) -> PluginDiagnostic {
        match self {
            Self::HookNotDeclared { plugin_id, hook }
            | Self::InvalidUtf8 {
                plugin_id, hook, ..
            }
            | Self::InvalidConfig {
                plugin_id, hook, ..
            }
            | Self::InvalidAdvancedValidatorConfig {
                plugin_id, hook, ..
            }
            | Self::TransformFailed {
                plugin_id, hook, ..
            } => PluginDiagnostic::error(self.to_string())
                .for_plugin(plugin_id.clone())
                .for_hook(*hook),
        }
    }
}

fn bundled_plugin(
    id: &str,
    name: &str,
    description: &str,
    hooks: &[HookKind],
    config_schema: ConfigSchemaMetadata,
    kind: BundledPluginKind,
) -> BundledPlugin {
    let manifest = PluginManifest {
        manifest_version: 1,
        id: id.to_owned(),
        name: name.to_owned(),
        version: Version::new(0, 1, 0),
        description: description.to_owned(),
        provider: "CorreoMQTT".to_owned(),
        license: "GPL-3.0-or-later".to_owned(),
        compatible_correomqtt: VersionReq::parse(">=0.1.0, <1.0.0")
            .expect("bundled compatibility requirement is valid"),
        capabilities: CapabilityGrants {
            hooks: hooks.to_vec(),
            host: Default::default(),
        },
        entrypoints: hooks
            .iter()
            .map(|hook| PluginEntrypoint {
                hook: *hook,
                export: bundled_export_name(*hook).to_owned(),
            })
            .collect(),
        connection_header_actions: Vec::new(),
        themes: Vec::new(),
        config_schema: Some(config_schema),
    };
    manifest
        .validate()
        .expect("bundled plugin manifest is valid");
    BundledPlugin { manifest, kind }
}

fn validate_contains_string(
    config: Value,
    payload: Vec<u8>,
    plugin_id: &str,
) -> Result<MessageValidatorResponse, BundledPluginError> {
    let config = serde_json::from_value::<ContainsStringConfig>(config).map_err(|source| {
        BundledPluginError::InvalidConfig {
            plugin_id: plugin_id.to_owned(),
            hook: HookKind::MessageValidator,
            source,
        }
    })?;
    let payload = String::from_utf8_lossy(&payload);
    let matches = if config.case_sensitive {
        payload.contains(&config.text)
    } else {
        payload.to_lowercase().contains(&config.text.to_lowercase())
    };
    let result = if matches {
        ValidationResultDto::Valid
    } else {
        ValidationResultDto::Invalid {
            message: if config.case_sensitive {
                "Payload does not contain the configured text.".to_owned()
            } else {
                "Payload does not contain the configured text, ignoring case.".to_owned()
            },
        }
    };
    Ok(MessageValidatorResponse {
        abi_version: ABI_VERSION,
        result,
    })
}

fn validate_xml_xsd_message(config: Value, payload: &[u8]) -> MessageValidatorResponse {
    let result = match validate_xml_xsd(config, payload) {
        XmlXsdValidation::Valid => ValidationResultDto::Valid,
        XmlXsdValidation::Invalid { message } => ValidationResultDto::Invalid { message },
    };
    MessageValidatorResponse {
        abi_version: ABI_VERSION,
        result,
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
struct ContainsStringConfig {
    text: String,
    case_sensitive: bool,
}

impl Default for ContainsStringConfig {
    fn default() -> Self {
        Self {
            text: String::new(),
            case_sensitive: true,
        }
    }
}

fn empty_config_schema() -> ConfigSchemaMetadata {
    ConfigSchemaMetadata {
        schema_version: 1,
        document: json!({
            "type": "object",
            "additionalProperties": false
        }),
    }
}

fn contains_string_config_schema() -> ConfigSchemaMetadata {
    ConfigSchemaMetadata {
        schema_version: 1,
        document: json!({
            "type": "object",
            "required": ["text"],
            "properties": {
                "text": { "type": "string" },
                "case_sensitive": { "type": "boolean", "default": true }
            },
            "additionalProperties": false
        }),
    }
}

fn xml_xsd_config_schema() -> ConfigSchemaMetadata {
    ConfigSchemaMetadata {
        schema_version: 1,
        document: config_schema_document(),
    }
}

fn bundled_export_name(hook: HookKind) -> &'static str {
    match hook {
        HookKind::OutgoingMessageTransform => "builtin_outgoing_message_transform",
        HookKind::IncomingMessageTransform => "builtin_incoming_message_transform",
        HookKind::MessageValidator => "builtin_message_validator",
        HookKind::DetailByteTransform => "builtin_detail_byte_transform",
        HookKind::DetailFormatter => "builtin_detail_formatter",
        HookKind::PayloadHighlighter => "builtin_payload_highlighter",
    }
}

fn supported_builtin(
    legacy_plugin_id: &'static str,
    replacement_plugin_id: &'static str,
) -> LegacyPluginReplacementDecision {
    supported(
        legacy_plugin_id,
        replacement_plugin_id,
        "Covered by a bundled Rust replacement for the MVP hook surface.",
    )
}

fn supported(
    legacy_plugin_id: &'static str,
    replacement_plugin_id: &'static str,
    reason: &'static str,
) -> LegacyPluginReplacementDecision {
    LegacyPluginReplacementDecision {
        legacy_plugin_id,
        status: LegacyPluginReplacementStatus::Supported,
        replacement_plugin_id: Some(replacement_plugin_id),
        reason,
    }
}
