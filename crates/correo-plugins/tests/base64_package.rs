use correo_plugins::{
    HookInvocation, HookKind, HookOutput, HostSurface, IncomingMessageTransformRequest, MessageDto,
    MessageTransformOutcomeDto, OutgoingMessageTransformRequest, PluginPackage, PluginRegistry,
};
use semver::Version;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

#[test]
fn base64_wasm_package_declares_minimal_capabilities_and_runs_supported_hooks() {
    let package_dir = build_base64_package();
    let package = PluginPackage::load(package_dir.path()).unwrap();
    let manifest = package.manifest();

    assert_eq!(manifest.id, "org.correomqtt.plugins.base64");
    assert_eq!(
        manifest
            .capabilities
            .hooks
            .iter()
            .copied()
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([
            HookKind::OutgoingMessageTransform,
            HookKind::IncomingMessageTransform,
        ])
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

    let outgoing = plugin
        .dispatch(HookInvocation::OutgoingMessageTransform(
            OutgoingMessageTransformRequest::new(MessageDto::new("demo/topic", b"hello")),
        ))
        .unwrap();
    assert_payload(outgoing, b"aGVsbG8=");

    let incoming = plugin
        .dispatch(HookInvocation::IncomingMessageTransform(
            IncomingMessageTransformRequest::new(MessageDto::new(
                "demo/topic",
                b"aGVsbG8=".to_vec(),
            )),
        ))
        .unwrap();
    assert_payload(incoming, b"hello");

    let invalid_incoming = plugin
        .dispatch(HookInvocation::IncomingMessageTransform(
            IncomingMessageTransformRequest::new(MessageDto::new(
                "demo/topic",
                b"not base64!".to_vec(),
            )),
        ))
        .unwrap();
    match invalid_incoming {
        HookOutput::IncomingMessageTransform(response) => {
            assert_eq!(response.outcome, MessageTransformOutcomeDto::Unchanged);
        }
        other => panic!("unexpected output: {other:?}"),
    }
}

fn build_base64_package() -> TempDir {
    let workspace = workspace_root();
    let plugin_crate = workspace.join("crates/correo-plugin-base64");
    let package_dir = TempDir::new().unwrap();
    let target_dir = TempDir::new().unwrap();

    let output = Command::new(env!("CARGO"))
        .current_dir(&workspace)
        .env("CARGO_TARGET_DIR", target_dir.path())
        .args([
            "build",
            "-p",
            "correo-plugin-base64",
            "--target",
            "wasm32-unknown-unknown",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "failed to build base64 WASM plugin:\nstdout:\n{}\nstderr:\n{}",
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
            .join("wasm32-unknown-unknown/debug/correo_plugin_base64.wasm"),
        package_dir.path().join("plugin.wasm"),
    )
    .unwrap();
    package_dir
}

fn assert_payload(output: HookOutput, expected: &[u8]) {
    let outcome = match output {
        HookOutput::OutgoingMessageTransform(response) => response.outcome,
        HookOutput::IncomingMessageTransform(response) => response.outcome,
        other => panic!("unexpected output: {other:?}"),
    };

    match outcome {
        MessageTransformOutcomeDto::Replace { message } => {
            assert_eq!(message.payload, expected);
        }
        other => panic!("unexpected outcome: {other:?}"),
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
