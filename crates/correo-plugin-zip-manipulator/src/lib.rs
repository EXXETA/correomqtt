use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::Deserialize;
#[cfg(target_arch = "wasm32")]
use serde::Serialize;
use serde_json::Value;
use std::io::{Read, Write};

#[cfg(target_arch = "wasm32")]
const ABI_VERSION: u16 = 1;
pub const CONFIGURATION_DESCRIPTION: &str = "Gzip-compresses outgoing payloads and decompresses incoming gzip payloads for matching topics.";
const GZIP_CONTENT_TYPE: &str = "application/gzip";
const GZIP_MAGIC: [u8; 2] = [0x1f, 0x8b];
pub const DEFAULT_MAX_INPUT_BYTES: usize = 1024 * 1024;
pub const DEFAULT_MAX_OUTPUT_BYTES: usize = 4 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZipOperation {
    Zip,
    Unzip,
}

impl Default for ZipOperation {
    fn default() -> Self {
        Self::Unzip
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZipLimits {
    pub max_input_bytes: usize,
    pub max_output_bytes: usize,
}

impl Default for ZipLimits {
    fn default() -> Self {
        Self {
            max_input_bytes: DEFAULT_MAX_INPUT_BYTES,
            max_output_bytes: DEFAULT_MAX_OUTPUT_BYTES,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetailTransformResult {
    pub bytes: Vec<u8>,
    pub content_type: Option<String>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ZipManipulatorError {
    #[error("ZIP manipulator config is invalid: {message}")]
    InvalidConfig { message: String },
    #[error("ZIP manipulator limit {field} must be greater than zero")]
    InvalidLimit { field: &'static str },
    #[error("ZIP manipulator input is empty")]
    EmptyInput,
    #[error("ZIP manipulator input is {actual} bytes; limit is {limit} bytes")]
    InputTooLarge { actual: usize, limit: usize },
    #[error("ZIP manipulator output exceeded {limit} bytes")]
    OutputTooLarge { limit: usize },
    #[error("ZIP manipulator I/O failed: {message}")]
    Io { message: String },
}

pub fn transform_detail_bytes(
    operation: ZipOperation,
    bytes: Vec<u8>,
    content_type: Option<String>,
    limits: ZipLimits,
) -> Result<DetailTransformResult, ZipManipulatorError> {
    validate_limits(limits)?;
    ensure_input_limit(bytes.len(), limits.max_input_bytes)?;

    match operation {
        ZipOperation::Zip => zip_detail_bytes(bytes, limits),
        ZipOperation::Unzip => unzip_detail_bytes(bytes, content_type, limits),
    }
}

pub fn transform_detail_bytes_from_config(
    config: Value,
    bytes: Vec<u8>,
    content_type: Option<String>,
) -> Result<DetailTransformResult, ZipManipulatorError> {
    let config = ZipConfig::from_value(config)?;
    transform_detail_bytes(config.operation, bytes, content_type, config.limits())
}

pub fn is_gzip_payload(bytes: &[u8]) -> bool {
    bytes.len() >= GZIP_MAGIC.len() && bytes[..2] == GZIP_MAGIC
}

fn zip_detail_bytes(
    bytes: Vec<u8>,
    limits: ZipLimits,
) -> Result<DetailTransformResult, ZipManipulatorError> {
    if bytes.is_empty() {
        return Err(ZipManipulatorError::EmptyInput);
    }

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&bytes).map_err(io_error)?;
    let compressed = encoder.finish().map_err(io_error)?;
    ensure_output_limit(compressed.len(), limits.max_output_bytes)?;
    Ok(DetailTransformResult {
        bytes: compressed,
        content_type: Some(GZIP_CONTENT_TYPE.to_owned()),
    })
}

fn unzip_detail_bytes(
    bytes: Vec<u8>,
    content_type: Option<String>,
    limits: ZipLimits,
) -> Result<DetailTransformResult, ZipManipulatorError> {
    if !is_gzip_payload(&bytes) {
        return Ok(DetailTransformResult {
            bytes,
            content_type,
        });
    }

    match decode_gzip(&bytes, limits.max_output_bytes) {
        Ok(decoded) => Ok(DetailTransformResult {
            bytes: decoded,
            content_type: None,
        }),
        Err(ZipManipulatorError::Io { .. }) => Ok(DetailTransformResult {
            bytes,
            content_type,
        }),
        Err(error) => Err(error),
    }
}

fn decode_gzip(bytes: &[u8], max_output_bytes: usize) -> Result<Vec<u8>, ZipManipulatorError> {
    let mut decoder = GzDecoder::new(bytes);
    let mut decoded = Vec::new();
    let mut chunk = [0; 8192];

    loop {
        let read = decoder.read(&mut chunk).map_err(io_error)?;
        if read == 0 {
            break;
        }
        if decoded.len().saturating_add(read) > max_output_bytes {
            return Err(ZipManipulatorError::OutputTooLarge {
                limit: max_output_bytes,
            });
        }
        decoded.extend_from_slice(&chunk[..read]);
    }

    Ok(decoded)
}

fn validate_limits(limits: ZipLimits) -> Result<(), ZipManipulatorError> {
    if limits.max_input_bytes == 0 {
        return Err(ZipManipulatorError::InvalidLimit {
            field: "max_input_bytes",
        });
    }
    if limits.max_output_bytes == 0 {
        return Err(ZipManipulatorError::InvalidLimit {
            field: "max_output_bytes",
        });
    }
    Ok(())
}

fn ensure_input_limit(actual: usize, limit: usize) -> Result<(), ZipManipulatorError> {
    if actual > limit {
        Err(ZipManipulatorError::InputTooLarge { actual, limit })
    } else {
        Ok(())
    }
}

fn ensure_output_limit(actual: usize, limit: usize) -> Result<(), ZipManipulatorError> {
    if actual > limit {
        Err(ZipManipulatorError::OutputTooLarge { limit })
    } else {
        Ok(())
    }
}

fn io_error(error: std::io::Error) -> ZipManipulatorError {
    ZipManipulatorError::Io {
        message: error.to_string(),
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
struct ZipConfig {
    operation: ZipOperation,
    max_input_bytes: usize,
    max_output_bytes: usize,
}

impl ZipConfig {
    fn from_value(value: Value) -> Result<Self, ZipManipulatorError> {
        if value.is_null() {
            return Ok(Self::default());
        }
        serde_json::from_value(value).map_err(|error| ZipManipulatorError::InvalidConfig {
            message: error.to_string(),
        })
    }

    fn limits(&self) -> ZipLimits {
        ZipLimits {
            max_input_bytes: self.max_input_bytes,
            max_output_bytes: self.max_output_bytes,
        }
    }
}

impl Default for ZipConfig {
    fn default() -> Self {
        Self {
            operation: ZipOperation::Unzip,
            max_input_bytes: DEFAULT_MAX_INPUT_BYTES,
            max_output_bytes: DEFAULT_MAX_OUTPUT_BYTES,
        }
    }
}

#[cfg(target_arch = "wasm32")]
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

#[cfg(target_arch = "wasm32")]
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

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub extern "C" fn correo_detail_byte_transform(ptr: i32, len: i32) -> i64 {
    let Some(request) = read_request(ptr, len) else {
        return write_response(&DetailByteTransformResponse::unchanged(Vec::new(), None));
    };
    if request.abi_version != ABI_VERSION {
        return write_response(&DetailByteTransformResponse::unchanged(
            request.bytes,
            request.content_type,
        ));
    }

    let _ = &request.context;
    let response = match transform_detail_bytes_from_config(
        request.config,
        request.bytes.clone(),
        request.content_type.clone(),
    ) {
        Ok(result) => DetailByteTransformResponse {
            abi_version: ABI_VERSION,
            bytes: result.bytes,
            content_type: result.content_type,
        },
        Err(_) => DetailByteTransformResponse::unchanged(request.bytes, request.content_type),
    };
    write_response(&response)
}

#[cfg(target_arch = "wasm32")]
fn read_request(ptr: i32, len: i32) -> Option<DetailByteTransformRequest> {
    if ptr <= 0 || len <= 0 {
        return None;
    }

    let len = usize::try_from(len).ok()?;
    let bytes = unsafe { std::slice::from_raw_parts(ptr as *const u8, len) };
    serde_json::from_slice::<DetailByteTransformRequest>(bytes).ok()
}

#[cfg(target_arch = "wasm32")]
fn write_response(response: &DetailByteTransformResponse) -> i64 {
    let Ok(bytes) = serde_json::to_vec(response) else {
        return 0;
    };
    let bytes = bytes.into_boxed_slice();
    let len = bytes.len();
    let ptr = Box::into_raw(bytes) as *mut u8 as i32;
    pack_ptr_len(ptr, len)
}

#[cfg(target_arch = "wasm32")]
fn pack_ptr_len(ptr: i32, len: usize) -> i64 {
    ((ptr as u32 as u64) << 32 | len as u64) as i64
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Serialize, Deserialize)]
struct DetailByteTransformRequest {
    abi_version: u16,
    #[serde(default)]
    context: Value,
    #[serde(default)]
    config: Value,
    bytes: Vec<u8>,
    #[serde(default)]
    content_type: Option<String>,
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct DetailByteTransformResponse {
    abi_version: u16,
    bytes: Vec<u8>,
    #[serde(default)]
    content_type: Option<String>,
}

#[cfg(target_arch = "wasm32")]
impl DetailByteTransformResponse {
    fn unchanged(bytes: Vec<u8>, content_type: Option<String>) -> Self {
        Self {
            abi_version: ABI_VERSION,
            bytes,
            content_type,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zip_then_unzip_round_trips_payload() {
        let zipped = transform_detail_bytes(
            ZipOperation::Zip,
            b"payload".to_vec(),
            None,
            ZipLimits::default(),
        )
        .unwrap();
        assert!(is_gzip_payload(&zipped.bytes));
        assert_eq!(zipped.content_type.as_deref(), Some(GZIP_CONTENT_TYPE));

        let unzipped = transform_detail_bytes(
            ZipOperation::Unzip,
            zipped.bytes,
            zipped.content_type,
            ZipLimits::default(),
        )
        .unwrap();
        assert_eq!(unzipped.bytes, b"payload");
        assert_eq!(unzipped.content_type, None);
    }

    #[test]
    fn unzip_leaves_plain_or_corrupt_payload_unchanged() {
        let plain = b"not gzip".to_vec();
        let result = transform_detail_bytes(
            ZipOperation::Unzip,
            plain.clone(),
            Some("text/plain".to_owned()),
            ZipLimits::default(),
        )
        .unwrap();

        assert_eq!(result.bytes, plain);
        assert_eq!(result.content_type.as_deref(), Some("text/plain"));

        let corrupt = vec![GZIP_MAGIC[0], GZIP_MAGIC[1], 0, 1, 2, 3];
        let result = transform_detail_bytes(
            ZipOperation::Unzip,
            corrupt.clone(),
            None,
            ZipLimits::default(),
        )
        .unwrap();
        assert_eq!(result.bytes, corrupt);
    }

    #[test]
    fn unzip_enforces_expansion_limit() {
        let zipped = transform_detail_bytes(
            ZipOperation::Zip,
            b"payload".to_vec(),
            None,
            ZipLimits::default(),
        )
        .unwrap();
        let error = transform_detail_bytes(
            ZipOperation::Unzip,
            zipped.bytes,
            zipped.content_type,
            ZipLimits {
                max_input_bytes: DEFAULT_MAX_INPUT_BYTES,
                max_output_bytes: 3,
            },
        )
        .unwrap_err();

        assert_eq!(error, ZipManipulatorError::OutputTooLarge { limit: 3 });
    }
}
