use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

const ABI_VERSION: u16 = 1;
pub const CONFIGURATION_DESCRIPTION: &str =
    "Encodes outgoing payloads as Base64 and decodes incoming Base64 payloads for matching topics.";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct MessageDto {
    topic: String,
    payload: Vec<u8>,
    qos: Value,
    retained: bool,
    #[serde(default)]
    properties: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct MessageTransformRequest {
    abi_version: u16,
    #[serde(default)]
    context: Value,
    #[serde(default)]
    config: Value,
    message: MessageDto,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct MessageTransformResponse {
    abi_version: u16,
    outcome: MessageTransformOutcome,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum MessageTransformOutcome {
    Unchanged,
    Replace { message: MessageDto },
}

#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub extern "C" fn correomqtt_alloc(len: i32) -> i32 {
    if len <= 0 {
        return 0;
    }

    let Ok(len) = usize::try_from(len) else {
        return 0;
    };
    let mut bytes = Vec::<u8>::with_capacity(len);
    let ptr = bytes.as_mut_ptr();
    std::mem::forget(bytes);
    ptr as i32
}

#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub extern "C" fn correomqtt_dealloc(ptr: i32, len: i32) {
    if ptr <= 0 || len <= 0 {
        return;
    }

    let Ok(len) = usize::try_from(len) else {
        return;
    };
    unsafe {
        drop(Vec::from_raw_parts(ptr as *mut u8, 0, len));
    }
}

#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub extern "C" fn correo_outgoing_transform(ptr: i32, len: i32) -> i64 {
    let Some(mut request) = read_request(ptr, len) else {
        return write_response(&unchanged());
    };

    write_response(&transform_outgoing_request(&mut request))
}

#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub extern "C" fn correo_incoming_transform(ptr: i32, len: i32) -> i64 {
    let Some(mut request) = read_request(ptr, len) else {
        return write_response(&unchanged());
    };

    write_response(&transform_incoming_request(&mut request))
}

fn transform_outgoing_request(request: &mut MessageTransformRequest) -> MessageTransformResponse {
    let encoded = base64::engine::general_purpose::STANDARD.encode(&request.message.payload);
    request.message.payload = encoded.into_bytes();
    replace(request.message.clone())
}

fn transform_incoming_request(request: &mut MessageTransformRequest) -> MessageTransformResponse {
    match base64::engine::general_purpose::STANDARD.decode(&request.message.payload) {
        Ok(decoded) => {
            request.message.payload = decoded;
            replace(request.message.clone())
        }
        Err(_) => unchanged(),
    }
}

fn read_request(ptr: i32, len: i32) -> Option<MessageTransformRequest> {
    if ptr <= 0 || len <= 0 {
        return None;
    }

    let len = usize::try_from(len).ok()?;
    let bytes = unsafe { std::slice::from_raw_parts(ptr as *const u8, len) };
    let request = serde_json::from_slice::<MessageTransformRequest>(bytes).ok()?;
    if request.abi_version == ABI_VERSION {
        let _ = (&request.context, &request.config);
        Some(request)
    } else {
        None
    }
}

fn replace(message: MessageDto) -> MessageTransformResponse {
    MessageTransformResponse {
        abi_version: ABI_VERSION,
        outcome: MessageTransformOutcome::Replace { message },
    }
}

fn unchanged() -> MessageTransformResponse {
    MessageTransformResponse {
        abi_version: ABI_VERSION,
        outcome: MessageTransformOutcome::Unchanged,
    }
}

fn write_response(response: &MessageTransformResponse) -> i64 {
    let Ok(bytes) = serde_json::to_vec(response) else {
        return 0;
    };
    let bytes = bytes.into_boxed_slice();
    let len = bytes.len();
    let ptr = Box::into_raw(bytes) as *mut u8 as i32;
    pack_ptr_len(ptr, len)
}

fn pack_ptr_len(ptr: i32, len: usize) -> i64 {
    ((ptr as u32 as u64) << 32 | len as u64) as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[test]
    fn manifest_declares_only_base64_transform_hooks() {
        let manifest = toml::from_str::<TestManifest>(include_str!("../plugin.toml")).unwrap();

        assert_eq!(manifest.id, "org.correomqtt.plugins.base64");
        assert_eq!(
            manifest.capabilities.hooks,
            [
                "outgoing_message_transform".to_owned(),
                "incoming_message_transform".to_owned(),
            ]
        );
        assert!(!manifest.capabilities.host.filesystem);
        assert!(!manifest.capabilities.host.network);
        assert!(!manifest.capabilities.host.secrets);
        assert!(!manifest.capabilities.host.mqtt);
        let entrypoints = manifest
            .entrypoints
            .iter()
            .map(|entrypoint| (entrypoint.hook.as_str(), entrypoint.export.as_str()))
            .collect::<Vec<_>>();
        assert_eq!(
            entrypoints,
            [
                ("outgoing_message_transform", "correo_outgoing_transform"),
                ("incoming_message_transform", "correo_incoming_transform"),
            ]
        );
        assert!(manifest.config_schema.is_some());
    }

    #[test]
    fn outgoing_payload_is_base64_encoded() {
        let mut request = request(b"hello".to_vec());
        let response = transform_outgoing_request(&mut request);

        assert_replaced_payload(response, b"aGVsbG8=");
    }

    #[test]
    fn incoming_payload_is_base64_decoded_when_valid() {
        let mut request = request(b"aGVsbG8=".to_vec());
        let response = transform_incoming_request(&mut request);

        assert_replaced_payload(response, b"hello");
    }

    #[test]
    fn incoming_payload_is_unchanged_when_invalid_base64() {
        let mut request = request(b"not base64!".to_vec());
        let response = transform_incoming_request(&mut request);

        assert_eq!(response.outcome, MessageTransformOutcome::Unchanged);
    }

    fn request(payload: Vec<u8>) -> MessageTransformRequest {
        MessageTransformRequest {
            abi_version: ABI_VERSION,
            context: Value::Null,
            config: Value::Null,
            message: MessageDto {
                topic: "demo/topic".to_owned(),
                payload,
                qos: Value::String("at_most_once".to_owned()),
                retained: false,
                properties: BTreeMap::new(),
            },
        }
    }

    fn assert_replaced_payload(response: MessageTransformResponse, expected: &[u8]) {
        match response.outcome {
            MessageTransformOutcome::Replace { message } => {
                assert_eq!(message.payload, expected);
            }
            other => panic!("unexpected outcome: {other:?}"),
        }
    }

    #[derive(Debug, Deserialize)]
    struct TestManifest {
        id: String,
        capabilities: TestCapabilities,
        entrypoints: Vec<TestEntrypoint>,
        config_schema: Option<Value>,
    }

    #[derive(Debug, Deserialize)]
    struct TestCapabilities {
        hooks: Vec<String>,
        host: TestHostCapabilities,
    }

    #[derive(Debug, Deserialize)]
    struct TestHostCapabilities {
        filesystem: bool,
        network: bool,
        secrets: bool,
        mqtt: bool,
    }

    #[derive(Debug, Deserialize)]
    struct TestEntrypoint {
        hook: String,
        export: String,
    }
}
