use correo_plugins::{
    DetailByteTransformRequest, HookInvocation, HookKind, HookOutput, HostSurface, PluginPackage,
    PluginRegistry,
};
use semver::Version;
use serde_json::json;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

#[test]
fn zip_manipulator_wasm_package_declares_minimal_capabilities_and_runs_hook() {
    let package_dir = build_zip_package();
    let package = PluginPackage::load(package_dir.path()).unwrap();
    let manifest = package.manifest();

    assert_eq!(manifest.id, "org.correomqtt.plugins.zip-manipulator");
    assert_eq!(
        manifest
            .capabilities
            .hooks
            .iter()
            .copied()
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([HookKind::DetailByteTransform])
    );
    for surface in [
        HostSurface::Filesystem,
        HostSurface::Network,
        HostSurface::Secrets,
        HostSurface::Mqtt,
    ] {
        assert!(!manifest.capabilities.grants_host_surface(surface));
    }
    assert!(manifest.config_schema.is_some());

    let mut registry = PluginRegistry::new(Version::new(1, 0, 0)).unwrap();
    let plugin = registry.register_package(package).unwrap();

    let mut request = DetailByteTransformRequest::new(b"payload".to_vec());
    request.config = json!({ "operation": "zip" });
    let zipped = plugin
        .dispatch(HookInvocation::DetailByteTransform(request))
        .unwrap();
    let (zipped, content_type) = assert_detail_bytes(zipped);

    assert!(zipped.starts_with(&[0x1f, 0x8b]));
    assert_eq!(content_type.as_deref(), Some("application/gzip"));

    let mut request = DetailByteTransformRequest::new(zipped);
    request.content_type = content_type;
    request.config = json!({ "operation": "unzip" });
    let unzipped = plugin
        .dispatch(HookInvocation::DetailByteTransform(request))
        .unwrap();
    let (unzipped, content_type) = assert_detail_bytes(unzipped);

    assert_eq!(unzipped, b"payload");
    assert_eq!(content_type, None);
}

fn build_zip_package() -> TempDir {
    let workspace = workspace_root();
    let plugin_crate = workspace.join("crates/correo-plugin-zip-manipulator");
    let package_dir = TempDir::new().unwrap();
    let target_dir = TempDir::new().unwrap();

    let output = Command::new(env!("CARGO"))
        .current_dir(&workspace)
        .env("CARGO_TARGET_DIR", target_dir.path())
        .args([
            "build",
            "-p",
            "correo-plugin-zip-manipulator",
            "--target",
            "wasm32-unknown-unknown",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "failed to build zip-manipulator WASM plugin:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    fs::copy(
        plugin_crate.join("plugin.toml"),
        package_dir.path().join("plugin.toml"),
    )
    .unwrap();
    fs::copy(
        target_dir
            .path()
            .join("wasm32-unknown-unknown/debug/correo_plugin_zip_manipulator.wasm"),
        package_dir.path().join("plugin.wasm"),
    )
    .unwrap();
    package_dir
}

fn assert_detail_bytes(output: HookOutput) -> (Vec<u8>, Option<String>) {
    match output {
        HookOutput::DetailByteTransform(response) => (response.bytes, response.content_type),
        other => panic!("unexpected output: {other:?}"),
    }
}

fn workspace_root() -> PathBuf {
    ancestor_with_file(Path::new(env!("CARGO_MANIFEST_DIR")), "Cargo.lock")
        .expect("workspace root should contain Cargo.lock")
}

fn ancestor_with_file(start: &Path, file_name: &str) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|path| path.join(file_name).is_file())
        .map(Path::to_path_buf)
}
