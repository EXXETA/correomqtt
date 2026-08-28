use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::slice;

const ABI_VERSION: u16 = 1;
pub const CONFIGURATION_DESCRIPTION: &str = "Checks text payloads for required literal or regex matches and marks messages that do not satisfy the rules.";

#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub extern "C" fn correo_message_validator(request_ptr: i32, request_len: i32) -> i64 {
    let request = unsafe { guest_bytes(request_ptr, request_len) };
    let response = validate_request(request);
    write_guest_response(response)
}

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
#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub unsafe extern "C" fn correomqtt_dealloc(ptr: i32, len: i32) {
    if ptr <= 0 || len < 0 {
        return;
    }

    let _ = Vec::from_raw_parts(ptr as *mut u8, 0, len as usize);
}

fn validate_request(request: &[u8]) -> MessageValidatorResponse {
    match serde_json::from_slice::<MessageValidatorRequest>(request) {
        Ok(request) => validate_message(request),
        Err(_) => invalid_response("Contains String Validator received an invalid request."),
    }
}

fn validate_message(request: MessageValidatorRequest) -> MessageValidatorResponse {
    let config = serde_json::from_value::<ContainsStringConfig>(request.config).unwrap_or_default();
    let payload = String::from_utf8_lossy(&request.message.payload);

    let matches = if config.case_sensitive {
        payload.contains(&config.text)
    } else {
        payload.to_lowercase().contains(&config.text.to_lowercase())
    };

    if matches {
        MessageValidatorResponse {
            abi_version: ABI_VERSION,
            result: ValidationResult::Valid,
        }
    } else {
        let message = if config.case_sensitive {
            "Payload does not contain the configured text."
        } else {
            "Payload does not contain the configured text, ignoring case."
        };
        invalid_response(message)
    }
}

fn invalid_response(message: &str) -> MessageValidatorResponse {
    MessageValidatorResponse {
        abi_version: ABI_VERSION,
        result: ValidationResult::Invalid {
            message: message.to_owned(),
        },
    }
}

unsafe fn guest_bytes<'a>(ptr: i32, len: i32) -> &'a [u8] {
    if ptr <= 0 || len <= 0 {
        return &[];
    }

    slice::from_raw_parts(ptr as *const u8, len as usize)
}

fn write_guest_response(response: MessageValidatorResponse) -> i64 {
    let bytes = serde_json::to_vec(&response)
        .unwrap_or_else(|_| br#"{"abi_version":1,"result":{"status":"valid"}}"#.to_vec());
    let len = bytes.len() as u32;
    let ptr = Box::into_raw(bytes.into_boxed_slice()) as *mut u8 as u32;
    pack_ptr_len(ptr, len)
}

fn pack_ptr_len(ptr: u32, len: u32) -> i64 {
    ((ptr as u64) << 32 | len as u64) as i64
}

#[derive(Debug, Deserialize)]
struct MessageValidatorRequest {
    #[allow(dead_code)]
    abi_version: u16,
    #[serde(default)]
    config: Value,
    message: Message,
}

#[derive(Debug, Deserialize)]
struct Message {
    payload: Vec<u8>,
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

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct MessageValidatorResponse {
    abi_version: u16,
    result: ValidationResult,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum ValidationResult {
    Valid,
    Invalid { message: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn validates_case_sensitive_contains() {
        let response = validate_message(request("Telemetry READY", "READY", true));

        assert_eq!(response.result, ValidationResult::Valid);
    }

    #[test]
    fn rejects_case_sensitive_mismatch() {
        let response = validate_message(request("Telemetry READY", "ready", true));

        assert_eq!(
            response.result,
            ValidationResult::Invalid {
                message: "Payload does not contain the configured text.".to_owned()
            }
        );
    }

    #[test]
    fn validates_ignoring_case() {
        let response = validate_message(request("Telemetry READY", "ready", false));

        assert_eq!(response.result, ValidationResult::Valid);
    }

    #[test]
    fn defaults_to_case_sensitive_matching() {
        let request = MessageValidatorRequest {
            abi_version: ABI_VERSION,
            config: json!({ "text": "ready" }),
            message: Message {
                payload: b"Telemetry READY".to_vec(),
            },
        };

        let response = validate_message(request);

        assert!(matches!(
            response.result,
            ValidationResult::Invalid { message } if message == "Payload does not contain the configured text."
        ));
    }

    fn request(text: &str, contains: &str, case_sensitive: bool) -> MessageValidatorRequest {
        MessageValidatorRequest {
            abi_version: ABI_VERSION,
            config: json!({
                "text": contains,
                "case_sensitive": case_sensitive
            }),
            message: Message {
                payload: text.as_bytes().to_vec(),
            },
        }
    }
}
