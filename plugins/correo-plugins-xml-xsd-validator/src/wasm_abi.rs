use super::{validate_xml_xsd, XmlXsdValidation};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::slice;

const ABI_VERSION: u16 = 1;

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
        Ok(request) => validate_xml_xsd(request.config, &request.message.payload).into(),
        Err(_) => invalid_response("XML/XSD Validator received an invalid request."),
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

impl From<XmlXsdValidation> for MessageValidatorResponse {
    fn from(validation: XmlXsdValidation) -> Self {
        let result = match validation {
            XmlXsdValidation::Valid => ValidationResult::Valid,
            XmlXsdValidation::Invalid { message } => ValidationResult::Invalid { message },
        };
        Self {
            abi_version: ABI_VERSION,
            result,
        }
    }
}
