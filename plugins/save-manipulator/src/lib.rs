use serde::{Deserialize, Serialize};

const ABI_VERSION: u16 = 1;
pub const CONFIGURATION_DESCRIPTION: &str = "Writes matching payloads to files in the selected folder without changing the message contents.";
const SUGGESTED_FILE_NAME: &str = "correomqtt-payload.txt";
const DEFAULT_CONTENT_TYPE: &str = "text/plain";

#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub extern "C" fn correo_detail_byte_transform(ptr: i32, len: i32) -> i64 {
    let input = unsafe { guest_bytes(ptr, len) };
    let output = transform_request_json(input);
    write_guest_response(output)
}

#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub extern "C" fn correomqtt_alloc(len: i32) -> i32 {
    if len < 0 {
        return -1;
    }

    let mut bytes = Vec::<u8>::with_capacity(len as usize);
    let ptr = bytes.as_mut_ptr();
    std::mem::forget(bytes);
    ptr as i32
}

#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub extern "C" fn correomqtt_dealloc(ptr: i32, len: i32) {
    if ptr <= 0 || len <= 0 {
        return;
    }

    unsafe {
        drop(Vec::from_raw_parts(
            ptr as *mut u8,
            len as usize,
            len as usize,
        ));
    }
}

fn transform_request_json(input: &[u8]) -> Vec<u8> {
    let response = match serde_json::from_slice::<DetailByteTransformRequest>(input) {
        Ok(request) if request.abi_version == ABI_VERSION => response_for(request),
        _ => empty_response(),
    };
    serde_json::to_vec(&response).expect("save response serializes")
}

fn response_for(request: DetailByteTransformRequest) -> DetailByteTransformResponse {
    let content_type = request
        .content_type
        .or_else(|| Some(DEFAULT_CONTENT_TYPE.to_owned()));
    DetailByteTransformResponse {
        abi_version: ABI_VERSION,
        bytes: request.bytes.clone(),
        content_type: content_type.clone(),
        host_actions: vec![HostAction::SavePayload(SavePayloadAction {
            suggested_file_name: SUGGESTED_FILE_NAME.to_owned(),
            bytes: request.bytes,
            content_type,
        })],
    }
}

fn empty_response() -> DetailByteTransformResponse {
    DetailByteTransformResponse {
        abi_version: ABI_VERSION,
        bytes: Vec::new(),
        content_type: None,
        host_actions: Vec::new(),
    }
}

fn write_guest_response(output: Vec<u8>) -> i64 {
    let len = output.len();
    let ptr = correomqtt_alloc(len as i32);
    if ptr < 0 {
        return 0;
    }

    unsafe {
        std::ptr::copy_nonoverlapping(output.as_ptr(), ptr as *mut u8, len);
    }
    pack_ptr_len(ptr as u32, len as u32) as i64
}

unsafe fn guest_bytes<'a>(ptr: i32, len: i32) -> &'a [u8] {
    if ptr <= 0 || len <= 0 {
        return &[];
    }
    std::slice::from_raw_parts(ptr as *const u8, len as usize)
}

fn pack_ptr_len(ptr: u32, len: u32) -> u64 {
    ((ptr as u64) << 32) | len as u64
}

#[derive(Debug, Deserialize)]
struct DetailByteTransformRequest {
    abi_version: u16,
    bytes: Vec<u8>,
    #[serde(default)]
    content_type: Option<String>,
}

#[derive(Debug, Serialize)]
struct DetailByteTransformResponse {
    abi_version: u16,
    bytes: Vec<u8>,
    #[serde(default)]
    content_type: Option<String>,
    #[serde(default)]
    host_actions: Vec<HostAction>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum HostAction {
    SavePayload(SavePayloadAction),
}

#[derive(Debug, Serialize)]
struct SavePayloadAction {
    suggested_file_name: String,
    bytes: Vec<u8>,
    #[serde(default)]
    content_type: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    #[test]
    fn requests_host_save_and_preserves_selected_payload() {
        let output = transform_request_json(
            &serde_json::to_vec(&json!({
                "abi_version": 1,
                "bytes": [104, 101, 108, 108, 111],
                "content_type": "text/plain"
            }))
            .unwrap(),
        );
        let value = serde_json::from_slice::<Value>(&output).unwrap();

        assert_eq!(value["abi_version"], 1);
        assert_eq!(value["bytes"], json!([104, 101, 108, 108, 111]));
        assert_eq!(value["host_actions"][0]["kind"], "save_payload");
        assert_eq!(
            value["host_actions"][0]["suggested_file_name"],
            SUGGESTED_FILE_NAME
        );
        assert_eq!(
            value["host_actions"][0]["bytes"],
            json!([104, 101, 108, 108, 111])
        );
    }

    #[test]
    fn invalid_request_returns_no_save_action() {
        let output = transform_request_json(br#"{"abi_version":2,"bytes":[1]}"#);
        let value = serde_json::from_slice::<Value>(&output).unwrap();

        assert_eq!(value["abi_version"], 1);
        assert_eq!(value["bytes"], json!([]));
        assert_eq!(value["host_actions"], json!([]));
    }
}
