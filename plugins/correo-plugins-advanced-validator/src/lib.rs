use serde::Deserialize;
#[cfg(target_arch = "wasm32")]
use serde::Serialize;
use serde_json::Value;
#[cfg(target_arch = "wasm32")]
use std::slice;
use thiserror::Error;

pub const ABI_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdvancedValidation {
    Valid,
    Invalid { message: String },
}

pub fn validate_payload(
    config: Value,
    payload: &[u8],
) -> Result<AdvancedValidation, AdvancedValidatorError> {
    let config =
        serde_json::from_value::<RuleNode>(config).map_err(AdvancedValidatorError::DecodeConfig)?;
    let payload = String::from_utf8_lossy(payload);

    if config.matches_and(&payload)? {
        Ok(AdvancedValidation::Valid)
    } else {
        Ok(AdvancedValidation::Invalid {
            message: "Advanced validator composition rejected the payload.".to_owned(),
        })
    }
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub extern "C" fn correo_message_validator(request_ptr: i32, request_len: i32) -> i64 {
    let request = unsafe { guest_bytes(request_ptr, request_len) };
    let response = validate_request(request);
    write_guest_response(response)
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub extern "C" fn correomqtt_alloc(len: i32) -> i32 {
    if len < 0 {
        return 0;
    }

    let mut bytes = Vec::<u8>::with_capacity(len as usize);
    let ptr = bytes.as_mut_ptr();
    std::mem::forget(bytes);
    ptr as i32
}

/// # Safety
/// `ptr` and `len` must originate from a matching `correomqtt_alloc` call;
/// the memory is reclaimed here and must not be accessed afterwards.
#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub unsafe extern "C" fn correomqtt_dealloc(ptr: i32, len: i32) {
    if ptr <= 0 || len < 0 {
        return;
    }

    unsafe {
        drop(Vec::from_raw_parts(ptr as *mut u8, 0, len as usize));
    }
}

#[cfg(target_arch = "wasm32")]
fn validate_request(request: &[u8]) -> MessageValidatorResponse {
    match serde_json::from_slice::<MessageValidatorRequest>(request) {
        Ok(request) if request.abi_version == ABI_VERSION => {
            match validate_payload(request.config, &request.message.payload) {
                Ok(AdvancedValidation::Valid) => valid_response(),
                Ok(AdvancedValidation::Invalid { message }) => invalid_response(message),
                Err(error) => invalid_response(error.to_string()),
            }
        }
        _ => invalid_response("Advanced Validator received an invalid request."),
    }
}

#[cfg(target_arch = "wasm32")]
unsafe fn guest_bytes<'a>(ptr: i32, len: i32) -> &'a [u8] {
    if ptr <= 0 || len <= 0 {
        return &[];
    }

    unsafe { slice::from_raw_parts(ptr as *const u8, len as usize) }
}

#[cfg(target_arch = "wasm32")]
fn write_guest_response(response: MessageValidatorResponse) -> i64 {
    let bytes = serde_json::to_vec(&response)
        .unwrap_or_else(|_| br#"{"abi_version":1,"result":{"status":"valid"}}"#.to_vec());
    let len = bytes.len() as u32;
    let ptr = Box::into_raw(bytes.into_boxed_slice()) as *mut u8 as u32;
    pack_ptr_len(ptr, len)
}

#[cfg(target_arch = "wasm32")]
fn pack_ptr_len(ptr: u32, len: u32) -> i64 {
    ((ptr as u64) << 32 | len as u64) as i64
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct RuleNode {
    and: Vec<RuleNode>,
    or: Vec<RuleNode>,
    extensions: Vec<ValidatorExtension>,
}

impl RuleNode {
    fn matches_and(&self, payload: &str) -> Result<bool, AdvancedValidatorError> {
        let mut and_matches = true;
        for node in &self.and {
            and_matches &= node.matches_and(payload)?;
        }

        let mut or_matches = self.or.is_empty();
        for node in &self.or {
            or_matches |= node.matches_or(payload)?;
        }

        let mut extension_matches = true;
        for extension in &self.extensions {
            extension_matches &= extension.matches(payload)?;
        }

        Ok(and_matches && or_matches && extension_matches)
    }

    fn matches_or(&self, payload: &str) -> Result<bool, AdvancedValidatorError> {
        for node in &self.and {
            if node.matches_and(payload)? {
                return Ok(true);
            }
        }
        for node in &self.or {
            if node.matches_or(payload)? {
                return Ok(true);
            }
        }
        for extension in &self.extensions {
            if extension.matches(payload)? {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
struct ValidatorExtension {
    #[serde(alias = "pluginId", alias = "plugin", alias = "name")]
    plugin_id: String,
    #[serde(alias = "extensionId", alias = "extension_id")]
    id: Option<String>,
    config: Value,
}

impl Default for ValidatorExtension {
    fn default() -> Self {
        Self {
            plugin_id: String::new(),
            id: None,
            config: Value::Null,
        }
    }
}

impl ValidatorExtension {
    fn matches(&self, payload: &str) -> Result<bool, AdvancedValidatorError> {
        if !is_contains_string_plugin(&self.plugin_id) {
            return Err(AdvancedValidatorError::UnsupportedExtension {
                plugin_id: self.plugin_id.clone(),
                extension_id: self.id.clone(),
            });
        }

        let config = serde_json::from_value::<ContainsStringConfig>(self.config.clone())
            .map_err(AdvancedValidatorError::DecodeExtensionConfig)?;
        let case_sensitive = match self.id.as_deref() {
            Some("caseSensitive" | "case_sensitive") => true,
            Some("ignoreCase" | "ignore_case") => false,
            Some(_) => {
                return Err(AdvancedValidatorError::UnsupportedExtension {
                    plugin_id: self.plugin_id.clone(),
                    extension_id: self.id.clone(),
                })
            }
            None => config.case_sensitive.unwrap_or(true),
        };

        if case_sensitive {
            Ok(payload.contains(&config.text))
        } else {
            Ok(payload.to_lowercase().contains(&config.text.to_lowercase()))
        }
    }
}

fn is_contains_string_plugin(plugin_id: &str) -> bool {
    matches!(
        plugin_id,
        "org.correomqtt.plugins.contains-string-validator"
            | "contains-string-validator"
            | "contains-string-validator-plugin"
    )
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct ContainsStringConfig {
    text: String,
    case_sensitive: Option<bool>,
}

#[derive(Debug, Error)]
pub enum AdvancedValidatorError {
    #[error("advanced validator config is invalid: {0}")]
    DecodeConfig(serde_json::Error),
    #[error("advanced validator extension config is invalid: {0}")]
    DecodeExtensionConfig(serde_json::Error),
    #[error("advanced validator does not support extension plugin {plugin_id}")]
    UnsupportedExtension {
        plugin_id: String,
        extension_id: Option<String>,
    },
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Deserialize)]
struct MessageValidatorRequest {
    abi_version: u16,
    #[serde(default)]
    config: Value,
    message: Message,
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Deserialize)]
struct Message {
    payload: Vec<u8>,
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct MessageValidatorResponse {
    abi_version: u16,
    result: ValidationResult,
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum ValidationResult {
    Valid,
    Invalid { message: String },
}

#[cfg(target_arch = "wasm32")]
fn valid_response() -> MessageValidatorResponse {
    MessageValidatorResponse {
        abi_version: ABI_VERSION,
        result: ValidationResult::Valid,
    }
}

#[cfg(target_arch = "wasm32")]
fn invalid_response(message: impl Into<String>) -> MessageValidatorResponse {
    MessageValidatorResponse {
        abi_version: ABI_VERSION,
        result: ValidationResult::Invalid {
            message: message.into(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn validates_and_or_composition_with_legacy_contains_extensions() {
        let config = json!({
            "and": [
                { "extensions": [extension("ignoreCase", "test")] },
                { "extensions": [extension("ignoreCase", "another")] }
            ],
            "or": [
                { "extensions": [extension("caseSensitive", "missing")] },
                { "extensions": [extension("caseSensitive", "okay")] }
            ]
        });

        assert_eq!(
            validate_payload(config, b"Test payload with another okay").unwrap(),
            AdvancedValidation::Valid
        );
    }

    #[test]
    fn rejects_when_any_and_group_fails() {
        let config = json!({
            "and": [
                { "extensions": [extension("ignoreCase", "test")] },
                { "extensions": [extension("ignoreCase", "missing")] }
            ]
        });

        assert!(matches!(
            validate_payload(config, b"Test payload").unwrap(),
            AdvancedValidation::Invalid { .. }
        ));
    }

    #[test]
    fn rejects_unsupported_extension_without_calling_host_surfaces() {
        let config = json!({
            "extensions": [{
                "pluginId": "external-schema-validator",
                "id": "default",
                "config": {}
            }]
        });

        assert!(matches!(
            validate_payload(config, b"payload").unwrap_err(),
            AdvancedValidatorError::UnsupportedExtension { .. }
        ));
    }

    fn extension(id: &str, text: &str) -> Value {
        json!({
            "pluginId": "contains-string-validator",
            "id": id,
            "config": { "text": text }
        })
    }
}
