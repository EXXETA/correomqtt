use correo_plugins::{
    DetailFormatDto, DetailFormatterRequest, HookInvocation, HookKind, HookOutput, HostSurface,
    PluginManifest, PluginPackage, WasmtimePluginRuntime,
};
use semver::Version;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

#[test]
fn json_format_manifest_declares_only_detail_formatter_without_host_caps() {
    let manifest = PluginManifest::from_toml_str(include_str!(
        "../../../plugins/correo-plugins-json-format/plugin.toml"
    ))
    .unwrap();

    assert_eq!(manifest.id, "org.correomqtt.plugins.json-format");
    assert_eq!(
        manifest.capabilities.hooks,
        vec![HookKind::DetailFormatter, HookKind::PayloadHighlighter]
    );
    assert_eq!(
        manifest
            .entrypoint_for(HookKind::DetailFormatter)
            .map(|entrypoint| entrypoint.export.as_str()),
        Some("correo_detail_formatter")
    );
    assert_eq!(
        manifest
            .entrypoint_for(HookKind::PayloadHighlighter)
            .map(|entrypoint| entrypoint.export.as_str()),
        Some("correo_payload_highlighter")
    );
    for surface in [
        HostSurface::Filesystem,
        HostSurface::Network,
        HostSurface::Secrets,
        HostSurface::Mqtt,
    ] {
        assert!(!manifest.capabilities.grants_host_surface(surface));
    }
}

#[test]
fn json_format_wasm_plugin_formats_detail_payloads() {
    let workspace = workspace_root();
    let wasm_path = build_json_format_wasm(&workspace);
    let package_dir = TempDir::new().unwrap();
    fs::copy(
        workspace.join("plugins/correo-plugins-json-format/plugin.toml"),
        package_dir.path().join("plugin.toml"),
    )
    .unwrap();
    fs::copy(wasm_path, package_dir.path().join("plugin.wasm")).unwrap();

    let package = PluginPackage::load(package_dir.path()).unwrap();
    let plugin = WasmtimePluginRuntime::default()
        .compile_package(package, &Version::new(0, 1, 0))
        .unwrap();

    let formatted = plugin
        .dispatch(HookInvocation::DetailFormatter(
            DetailFormatterRequest::new(br#"{"ok":true,"items":[1,2]}"#.to_vec()),
        ))
        .unwrap();
    assert_formatted_json(formatted);

    let fallback = plugin
        .dispatch(HookInvocation::DetailFormatter(
            DetailFormatterRequest::new(b"not json".to_vec()),
        ))
        .unwrap();
    assert_plain_text_warning(fallback);
}

fn build_json_format_wasm(workspace: &Path) -> PathBuf {
    let target_dir = TempDir::new().unwrap().keep();
    let status = Command::new(env!("CARGO"))
        .current_dir(workspace)
        .env("CARGO_TARGET_DIR", &target_dir)
        .args([
            "build",
            "-p",
            "correo-plugins-json-format",
            "--target",
            "wasm32-unknown-unknown",
            "--release",
        ])
        .status()
        .unwrap();
    assert!(
        status.success(),
        "failed to build JSON formatter WASM plugin"
    );

    target_dir
        .join("wasm32-unknown-unknown")
        .join("release")
        .join("correo_plugins_json_format.wasm")
}

fn assert_formatted_json(output: HookOutput) {
    match output {
        HookOutput::DetailFormatter(response) => {
            assert_eq!(response.output.format, DetailFormatDto::Json);
            assert!(response.output.text.contains("\"items\": ["));
            assert!(response.output.diagnostics.is_empty());
        }
        other => panic!("unexpected hook output: {other:?}"),
    }
}

fn assert_plain_text_warning(output: HookOutput) {
    match output {
        HookOutput::DetailFormatter(response) => {
            assert_eq!(response.output.format, DetailFormatDto::PlainText);
            assert_eq!(response.output.text, "not json");
            assert_eq!(response.output.diagnostics.len(), 1);
        }
        other => panic!("unexpected hook output: {other:?}"),
    }
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap()
        .to_path_buf()
}
