use correo_plugins::{
    HookDispatchError, HookKind, HostSurface, IntoPluginDiagnostic, MessageDto,
    MessageValidatorRequest, NoopHookFixture, PluginPackage, PluginRegistry, RegistryError,
    RuntimeLoadError, WasmFixtureHarness, WasmSandboxLimits, WasmtimePluginRuntime,
};
use semver::Version;
use std::fs;
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

#[test]
fn package_loader_accepts_mvp_layout_with_optional_assets() {
    let (_dir, package) = write_package(all_hooks_manifest(), minimal_wasm(), true);

    assert_eq!(
        package.manifest().id,
        "org.correomqtt.plugins.runtime_fixture"
    );
    assert!(package.wasm_path().ends_with("plugin.wasm"));
    assert!(package.assets_path().is_some());
}

#[test]
fn package_abi_defaults_to_v1_and_selects_v2() {
    let (_legacy_dir, legacy) = write_package(all_hooks_manifest(), minimal_wasm(), false);
    assert_eq!(legacy.abi(), correo_plugins::PluginAbi::V1);

    let v2_manifest = all_hooks_manifest().replace(
        "manifest_version = 1",
        "manifest_version = 1\nabi_version = 2",
    );
    let (_v2_dir, v2) = write_package(v2_manifest, minimal_wasm(), false);
    assert_eq!(v2.abi(), correo_plugins::PluginAbi::V2);
}

#[test]
fn package_abi_rejects_malformed_and_unsupported_selectors() {
    for (selector, expected) in [
        (
            "abi_version = \"two\"",
            "plugin package abi_version is malformed",
        ),
        (
            "abi_version = 3",
            "plugin package abi_version 3 is unsupported",
        ),
    ] {
        let dir = TempDir::new().unwrap();
        let manifest = all_hooks_manifest().replace(
            "manifest_version = 1",
            &format!("manifest_version = 1\n{selector}"),
        );
        fs::write(dir.path().join("plugin.toml"), manifest).unwrap();
        fs::write(dir.path().join("plugin.wasm"), minimal_wasm()).unwrap();
        assert_eq!(
            PluginPackage::load(dir.path()).unwrap_err().to_string(),
            expected
        );
    }
}

#[test]
fn v2_package_dispatches_a_v2_wasm_response_and_rejects_v1_invocation() {
    let manifest = message_validator_manifest().replace(
        "manifest_version = 1",
        "manifest_version = 1\nabi_version = 2",
    );
    let response = correo_plugins::TransportMessageValidatorResponse {
        abi_version: correo_plugins::ABI_VERSION_V2,
        result: correo_plugins::ValidationResultDto::Valid,
    };
    let (_dir, package) = write_package(
        manifest,
        static_response_validator_wasm(&serde_json::to_vec(&response).unwrap(), 1),
        false,
    );
    let runtime = WasmtimePluginRuntime::default();
    let plugin = runtime
        .compile_package(package, &Version::new(1, 0, 0))
        .unwrap();
    let v2 = correo_plugins::HookInvocation::TransportMessageValidator(
        correo_plugins::TransportMessageValidatorRequest {
            abi_version: correo_plugins::ABI_VERSION_V2,
            context: Default::default(),
            config: serde_json::Value::Null,
            input: correo_plugins::TransportHookInputDto {
                message: correo_plugins::TransportMessageDto {
                    address: "orders.created".to_owned(),
                    body: vec![1, 2, 3],
                    delivery: correo_plugins::DeliveryGuaranteeDto::AtLeastOnce,
                    metadata: Default::default(),
                },
                consumer: None,
                capabilities: Default::default(),
            },
        },
    );
    assert!(matches!(
        plugin.dispatch(v2).unwrap(),
        correo_plugins::HookOutput::TransportMessageValidator(_)
    ));
    let error = plugin.dispatch(message_validator_invocation()).unwrap_err();
    assert!(matches!(
        error,
        HookDispatchError::AbiVersionMismatch {
            expected: 2,
            found: 1,
            ..
        }
    ));
}

#[test]
fn registry_dispatches_noop_fixture_for_every_supported_hook() {
    let fixtures = WasmFixtureHarness::new(fixture_root())
        .load_all_noop_fixtures()
        .unwrap();
    let (_dir, package) = write_package(all_hooks_manifest(), noop_fixture_wasm(&fixtures), false);
    let mut registry = PluginRegistry::new(Version::new(1, 0, 0)).unwrap();
    let plugin = registry.register_package(package).unwrap();

    for fixture in fixtures {
        let output = plugin.dispatch(fixture.invocation()).unwrap();
        assert_eq!(output, fixture.expected_output());
    }
}

#[test]
fn registry_rejects_manifest_incompatible_with_current_app_version() {
    let (_dir, package) = write_package(incompatible_manifest(), minimal_wasm(), false);
    let mut registry = PluginRegistry::new(Version::new(1, 0, 0)).unwrap();
    let error = registry.register_package(package).unwrap_err();

    assert!(matches!(
        error,
        RegistryError::Runtime(RuntimeLoadError::IncompatibleCorreoVersion { .. })
    ));
    assert_eq!(
        error.diagnostic().severity,
        correo_plugins::PluginDiagnosticSeverity::Error
    );
}

#[test]
fn registry_rejects_unsupported_host_capabilities() {
    let (_dir, package) = write_package(
        host_capability_manifest(HostSurface::Filesystem),
        minimal_wasm(),
        false,
    );
    let mut registry = PluginRegistry::new(Version::new(1, 0, 0)).unwrap();
    let error = registry.register_package(package).unwrap_err();

    assert!(matches!(
        error,
        RegistryError::Runtime(RuntimeLoadError::UnsupportedHostCapability {
            surface: HostSurface::Filesystem,
            ..
        })
    ));
}

#[test]
fn registry_rejects_wasm_imports_before_plugin_runs() {
    let (_dir, package) = write_package(all_hooks_manifest(), importing_wasm(), false);
    let mut registry = PluginRegistry::new(Version::new(1, 0, 0)).unwrap();
    let error = registry.register_package(package).unwrap_err();

    assert!(matches!(
        error,
        RegistryError::Runtime(RuntimeLoadError::HostImportDenied { .. })
    ));
}

#[test]
fn registry_rejects_missing_entrypoint_export_before_plugin_runs() {
    let (_dir, package) = write_package(all_hooks_manifest(), minimal_wasm(), false);
    let mut registry = PluginRegistry::new(Version::new(1, 0, 0)).unwrap();
    let error = registry.register_package(package).unwrap_err();

    assert!(matches!(
        error,
        RegistryError::Runtime(RuntimeLoadError::MissingEntrypointExport { .. })
    ));
}

#[test]
fn registry_rejects_duplicate_plugin_ids() {
    let mut registry = PluginRegistry::new(Version::new(1, 0, 0)).unwrap();
    let (_first_dir, first) = write_package(message_validator_manifest(), validator_wasm(), false);
    let (_second_dir, second) =
        write_package(message_validator_manifest(), validator_wasm(), false);

    registry.register_package(first).unwrap();
    let error = registry.register_package(second).unwrap_err();

    assert!(matches!(error, RegistryError::DuplicatePluginId { .. }));
}

#[test]
fn sandbox_fuel_limit_isolated_as_typed_dispatch_error() {
    let limits = WasmSandboxLimits {
        max_fuel: 1_000,
        ..Default::default()
    };
    let runtime = WasmtimePluginRuntime::new(limits).unwrap();
    let (_dir, package) = write_package(
        message_validator_manifest(),
        looping_validator_wasm(),
        false,
    );
    let plugin = runtime
        .compile_package(package, &Version::new(1, 0, 0))
        .unwrap();

    let error = plugin.dispatch(message_validator_invocation()).unwrap_err();

    assert!(matches!(
        error,
        HookDispatchError::FuelExhausted {
            hook: HookKind::MessageValidator
        }
    ));
}

#[test]
fn cancellation_token_interrupts_running_hook() {
    let limits = WasmSandboxLimits {
        max_fuel: u64::MAX / 2,
        ..Default::default()
    };
    let runtime = WasmtimePluginRuntime::new(limits).unwrap();
    let token = runtime.cancellation_token();
    let (_dir, package) = write_package(
        message_validator_manifest(),
        looping_validator_wasm(),
        false,
    );
    let plugin = runtime
        .compile_package(package, &Version::new(1, 0, 0))
        .unwrap();

    let worker_token = token.clone();
    let worker = thread::spawn(move || {
        plugin.dispatch_with_cancel(message_validator_invocation(), &worker_token)
    });
    thread::sleep(Duration::from_millis(20));
    token.cancel();
    let error = worker.join().unwrap().unwrap_err();

    assert!(matches!(
        error,
        HookDispatchError::Cancelled {
            hook: HookKind::MessageValidator
        }
    ));
}

#[test]
fn registry_rejects_initial_memory_over_limit() {
    let limits = WasmSandboxLimits {
        max_memory_bytes: 64 * 1024,
        ..Default::default()
    };
    let runtime = WasmtimePluginRuntime::new(limits).unwrap();
    let mut registry = PluginRegistry::with_runtime(Version::new(1, 0, 0), runtime);
    let (_dir, package) = write_package(
        message_validator_manifest(),
        two_page_validator_wasm(),
        false,
    );
    let error = registry.register_package(package).unwrap_err();

    assert!(matches!(
        error,
        RegistryError::Runtime(RuntimeLoadError::InitialMemoryTooLarge { .. })
    ));
}

fn write_package(manifest: String, wasm: Vec<u8>, with_assets: bool) -> (TempDir, PluginPackage) {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("plugin.toml"), manifest).unwrap();
    fs::write(dir.path().join("plugin.wasm"), wasm).unwrap();
    if with_assets {
        fs::create_dir(dir.path().join("assets")).unwrap();
    }
    let package = PluginPackage::load(dir.path()).unwrap();
    (dir, package)
}

fn all_hooks_manifest() -> String {
    manifest_for(
        &[
            (
                HookKind::OutgoingMessageTransform,
                "correo_outgoing_transform",
            ),
            (
                HookKind::IncomingMessageTransform,
                "correo_incoming_transform",
            ),
            (HookKind::MessageValidator, "correo_message_validator"),
            (
                HookKind::DetailByteTransform,
                "correo_detail_byte_transform",
            ),
            (HookKind::DetailFormatter, "correo_detail_formatter"),
        ],
        ">=1.0.0, <2.0.0",
        None,
    )
}

fn message_validator_manifest() -> String {
    manifest_for(
        &[(HookKind::MessageValidator, "correo_message_validator")],
        ">=1.0.0, <2.0.0",
        None,
    )
}

fn incompatible_manifest() -> String {
    manifest_for(
        &[(HookKind::MessageValidator, "correo_message_validator")],
        ">=2.0.0",
        None,
    )
}

fn host_capability_manifest(surface: HostSurface) -> String {
    manifest_for(
        &[(HookKind::MessageValidator, "correo_message_validator")],
        ">=1.0.0, <2.0.0",
        Some(surface),
    )
}

fn manifest_for(
    entrypoints: &[(HookKind, &str)],
    compatible: &str,
    host_surface: Option<HostSurface>,
) -> String {
    let hooks = entrypoints
        .iter()
        .map(|(hook, _)| format!("\"{}\"", hook_name(*hook)))
        .collect::<Vec<_>>()
        .join(", ");
    let host = |surface| {
        if host_surface == Some(surface) {
            "true"
        } else {
            "false"
        }
    };
    let mut manifest = format!(
        r#"manifest_version = 1
id = "org.correomqtt.plugins.runtime_fixture"
name = "Runtime Fixture"
version = "0.1.0"
description = "Synthetic runtime fixture."
provider = "CorreoMQTT"
license = "GPL-3.0-or-later"
compatible_correomqtt = "{compatible}"

[capabilities]
hooks = [{hooks}]

[capabilities.host]
filesystem = {}
network = {}
secrets = {}
mqtt = {}
"#,
        host(HostSurface::Filesystem),
        host(HostSurface::Network),
        host(HostSurface::Secrets),
        host(HostSurface::Mqtt)
    );
    for (hook, export) in entrypoints {
        manifest.push_str(&format!(
            "\n[[entrypoints]]\nhook = \"{}\"\nexport = \"{}\"\n",
            hook_name(*hook),
            export
        ));
    }
    manifest
}

fn noop_fixture_wasm(fixtures: &[NoopHookFixture]) -> Vec<u8> {
    let mut wat = allocator_module(1);
    let mut offset = 1024u32;
    for fixture in fixtures {
        let bytes = expected_response_bytes(fixture);
        wat.push_str(&format!(
            "(data (i32.const {offset}) \"{}\")\n",
            wat_escape(&bytes)
        ));
        wat.push_str(&format!(
            "(func (export \"{}\") (param i32 i32) (result i64) i64.const {})\n",
            export_name(fixture.hook()),
            pack_ptr_len(offset, bytes.len() as u32)
        ));
        offset += bytes.len() as u32 + 16;
    }
    wat.push(')');
    wat::parse_str(&wat).unwrap()
}

fn validator_wasm() -> Vec<u8> {
    let response = serde_json::to_vec(&correo_plugins::MessageValidatorResponse::valid()).unwrap();
    static_response_validator_wasm(&response, 1)
}

fn looping_validator_wasm() -> Vec<u8> {
    let mut wat = allocator_module(1);
    wat.push_str(
        r#"(func (export "correo_message_validator") (param i32 i32) (result i64)
  (loop $again
    br $again)
  i64.const 0)
)"#,
    );
    wat::parse_str(&wat).unwrap()
}

fn two_page_validator_wasm() -> Vec<u8> {
    let response = serde_json::to_vec(&correo_plugins::MessageValidatorResponse::valid()).unwrap();
    static_response_validator_wasm(&response, 2)
}

fn minimal_wasm() -> Vec<u8> {
    let mut wat = allocator_module(1);
    wat.push(')');
    wat::parse_str(&wat).unwrap()
}

fn importing_wasm() -> Vec<u8> {
    wat::parse_str(
        r#"(module
  (import "env" "host_read_file" (func $host_read_file))
  (memory (export "memory") 1 1)
  (func (export "correomqtt_alloc") (param i32) (result i32) i32.const 2048)
)"#,
    )
    .unwrap()
}
