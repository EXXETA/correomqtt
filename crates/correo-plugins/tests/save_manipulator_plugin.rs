use correo_plugins::{
    DetailByteTransformRequest, DetailByteTransformResponse, HookInvocation, HookKind, HookOutput,
    HostActionDto, HostSurface, PluginManifest, PluginPackage, PluginRegistry,
    SavePayloadActionDto, ABI_VERSION, SAVE_MANIPULATOR_ID,
};
use semver::Version;
use std::fs;
use tempfile::TempDir;

const MANIFEST: &str = include_str!("../../../plugins/save-manipulator/plugin.toml");

#[test]
fn save_manipulator_manifest_declares_bounded_save_surface() {
    let manifest = PluginManifest::from_toml_str(MANIFEST).unwrap();

    assert_eq!(manifest.id, SAVE_MANIPULATOR_ID);
    assert_eq!(
        manifest.capabilities.hooks,
        vec![HookKind::DetailByteTransform]
    );
    assert!(manifest
        .capabilities
        .grants_host_surface(HostSurface::MessageSave));
    assert!(!manifest
        .capabilities
        .grants_host_surface(HostSurface::Filesystem));
    assert!(!manifest
        .capabilities
        .grants_host_surface(HostSurface::Network));
    assert!(!manifest
        .capabilities
        .grants_host_surface(HostSurface::Secrets));
    assert!(!manifest.capabilities.grants_host_surface(HostSurface::Mqtt));
    assert_eq!(
        manifest
            .entrypoint_for(HookKind::DetailByteTransform)
            .map(|entrypoint| entrypoint.export.as_str()),
        Some("correo_detail_byte_transform")
    );
}

#[test]
fn runtime_accepts_save_manipulator_response_for_detail_transform_hook() {
    let payload = b"payload".to_vec();
    let response = DetailByteTransformResponse {
        abi_version: ABI_VERSION,
        bytes: payload.clone(),
        content_type: Some("text/plain".to_owned()),
        host_actions: vec![HostActionDto::SavePayload(SavePayloadActionDto {
            suggested_file_name: "correomqtt-payload.txt".to_owned(),
            bytes: payload.clone(),
            content_type: Some("text/plain".to_owned()),
        })],
    };
    let (_dir, package) = write_package(static_detail_transform_wasm(&response));
    let mut registry = PluginRegistry::new(Version::new(1, 0, 0)).unwrap();
    let plugin = registry.register_package(package).unwrap();

    let output = plugin
        .dispatch(HookInvocation::DetailByteTransform(
            DetailByteTransformRequest::new(payload.clone()),
        ))
        .unwrap();

    match output {
        HookOutput::DetailByteTransform(output) => {
            assert_eq!(output.bytes, payload);
            assert_eq!(output.host_actions, response.host_actions);
        }
        other => panic!("unexpected output: {other:?}"),
    }
}

fn write_package(wasm: Vec<u8>) -> (TempDir, PluginPackage) {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("plugin.toml"), MANIFEST).unwrap();
    fs::write(dir.path().join("plugin.wasm"), wasm).unwrap();
    let package = PluginPackage::load(dir.path()).unwrap();
    (dir, package)
}

fn static_detail_transform_wasm(response: &DetailByteTransformResponse) -> Vec<u8> {
    let bytes = serde_json::to_vec(response).unwrap();
    let wat = format!(
        r#"(module
  (memory (export "memory") 1 1)
  (global $heap (mut i32) (i32.const 16384))
  (func (export "correomqtt_alloc") (param $len i32) (result i32)
    (local $ptr i32)
    global.get $heap
    local.set $ptr
    global.get $heap
    local.get $len
    i32.add
    global.set $heap
    local.get $ptr)
  (func (export "correomqtt_dealloc") (param i32) (param i32))
  (data (i32.const 1024) "{}")
  (func (export "correo_detail_byte_transform") (param i32 i32) (result i64)
    i64.const {})
)"#,
        wat_escape(&bytes),
        pack_ptr_len(1024, bytes.len() as u32)
    );
    wat::parse_str(&wat).unwrap()
}

fn pack_ptr_len(ptr: u32, len: u32) -> u64 {
    ((ptr as u64) << 32) | len as u64
}

fn wat_escape(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("\\{byte:02x}")).collect()
}
