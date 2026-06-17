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
fn xml_format_manifest_declares_only_detail_formatter_without_host_caps() {
    let manifest =
        PluginManifest::from_toml_str(include_str!("../../../plugins/xml-format/plugin.toml"))
            .unwrap();

    assert_eq!(manifest.id, "org.correomqtt.plugins.xml-format");
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
fn xml_format_wasm_plugin_formats_detail_payloads() {
    let workspace = workspace_root();
    let wasm_path = build_xml_format_wasm(&workspace);
    let package_dir = TempDir::new().unwrap();
    fs::copy(
        workspace.join("plugins/xml-format/plugin.toml"),
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
            DetailFormatterRequest::new(br#"<root><item id="1">ok</item></root>"#.to_vec()),
        ))
        .unwrap();
    assert_formatted_xml(formatted);

    let fallback = plugin
        .dispatch(HookInvocation::DetailFormatter(
            DetailFormatterRequest::new(b"not xml".to_vec()),
        ))
        .unwrap();
    assert_plain_text_warning(fallback);
}

fn build_xml_format_wasm(workspace: &Path) -> PathBuf {
    let target_dir = workspace.join("target/xml-format-plugin-test");
    fs::create_dir_all(&target_dir).unwrap();
    let status = Command::new(env!("CARGO"))
        .current_dir(workspace)
        .env("CARGO_TARGET_DIR", &target_dir)
        .args([
            "build",
            "-p",
            "correo-plugin-xml-format",
            "--target",
            "wasm32-unknown-unknown",
            "--release",
        ])
        .status()
        .unwrap();
    assert!(
        status.success(),
        "failed to build XML formatter WASM plugin"
    );

    target_dir
        .join("wasm32-unknown-unknown")
        .join("release")
        .join("correo_plugin_xml_format.wasm")
}

fn assert_formatted_xml(output: HookOutput) {
    match output {
        HookOutput::DetailFormatter(response) => {
            assert_eq!(response.output.format, DetailFormatDto::Xml);
            assert!(response.output.text.contains("  <item id=\"1\">"));
            assert!(response.output.diagnostics.is_empty());
        }
        other => panic!("unexpected hook output: {other:?}"),
    }
}

fn assert_plain_text_warning(output: HookOutput) {
    match output {
        HookOutput::DetailFormatter(response) => {
            assert_eq!(response.output.format, DetailFormatDto::PlainText);
            assert_eq!(response.output.text, "not xml");
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
