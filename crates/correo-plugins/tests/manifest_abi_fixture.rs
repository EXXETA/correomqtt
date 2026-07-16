use correo_plugins::{
    CapabilityGrants, DetailByteTransformRequest, DetailByteTransformResponse,
    DetailFormatterRequest, DetailFormatterResponse, HookKind, HostActionDto, HostSurface,
    IncomingMessageTransformRequest, IncomingMessageTransformResponse, ManifestError, MessageDto,
    MessageValidatorRequest, MessageValidatorResponse, NoopHookFixture,
    OutgoingMessageTransformRequest, OutgoingMessageTransformResponse, PluginManifest,
    SavePayloadActionDto, VersionedDto, WasmFixtureHarness, ABI_VERSION,
};
use semver::Version;
use serde::{de::DeserializeOwned, Serialize};
use std::collections::BTreeSet;
use std::fmt::Debug;
use std::path::PathBuf;

#[test]
fn parses_plugin_manifest_with_config_schema_metadata() {
    let manifest = PluginManifest::from_toml_str(include_str!("fixtures/plugin.toml")).unwrap();

    assert_eq!(manifest.manifest_version, 1);
    assert_eq!(manifest.id, "org.correomqtt.plugins.sample");
    assert_eq!(manifest.name, "Sample WASM Plugin");
    assert_eq!(manifest.version, Version::new(0, 1, 0));
    assert!(manifest
        .compatible_correomqtt
        .matches(&Version::new(1, 0, 0)));
    assert!(manifest
        .capabilities
        .grants_hook(HookKind::OutgoingMessageTransform));
    assert!(!manifest
        .capabilities
        .grants_host_surface(HostSurface::Filesystem));
    assert_eq!(
        manifest
            .entrypoint_for(HookKind::DetailFormatter)
            .map(|entrypoint| entrypoint.export.as_str()),
        Some("correo_detail_formatter")
    );

    let schema = manifest.config_schema.unwrap();
    assert_eq!(schema.schema_version, 1);
    assert_eq!(schema.document["type"].as_str(), Some("object"));
}

#[test]
fn manifest_rejects_entrypoint_without_hook_capability() {
    let error = PluginManifest::from_toml_str(
        r#"
manifest_version = 1
id = "org.correomqtt.plugins.bad"
name = "Bad Plugin"
version = "0.1.0"
description = "Entrypoint is not granted."
provider = "CorreoMQTT"
license = "GPL-3.0-or-later"
compatible_correomqtt = ">=0.1.0"

[capabilities]
hooks = ["incoming_message_transform"]

[[entrypoints]]
hook = "outgoing_message_transform"
export = "correo_outgoing_transform"
"#,
    )
    .unwrap_err();

    assert!(matches!(
        error,
        ManifestError::EntrypointCapabilityMissing {
            hook: HookKind::OutgoingMessageTransform
        }
    ));
}

#[test]
fn capability_grants_deny_host_surfaces_by_default() {
    let grants = CapabilityGrants::default();

    for surface in [
        HostSurface::Filesystem,
        HostSurface::MessageSave,
        HostSurface::Network,
        HostSurface::Secrets,
        HostSurface::Mqtt,
    ] {
        assert!(!grants.grants_host_surface(surface));
        assert!(grants.ensure_host_surface(surface).is_err());
    }
}

#[test]
fn hook_dtos_round_trip_as_versioned_json() {
    let message = MessageDto::new("fixture/topic", b"payload".to_vec());

    assert_versioned_json_round_trip(OutgoingMessageTransformRequest::new(message.clone()));
    assert_versioned_json_round_trip(OutgoingMessageTransformResponse::unchanged());
    assert_versioned_json_round_trip(IncomingMessageTransformRequest::new(message.clone()));
    assert_versioned_json_round_trip(IncomingMessageTransformResponse::unchanged());
    assert_versioned_json_round_trip(MessageValidatorRequest::new(message));
    assert_versioned_json_round_trip(MessageValidatorResponse::valid());
    assert_versioned_json_round_trip(DetailByteTransformRequest::new(b"{}".to_vec()));
    assert_versioned_json_round_trip(DetailByteTransformResponse::unchanged(b"{}".to_vec()));
    assert_versioned_json_round_trip(DetailFormatterRequest::new(b"{}".to_vec()));
    assert_versioned_json_round_trip(DetailFormatterResponse::plain_text("{}"));
}

#[test]
fn detail_byte_transform_response_can_request_host_save() {
    let response = DetailByteTransformResponse {
        abi_version: ABI_VERSION,
        bytes: b"payload".to_vec(),
        content_type: Some("text/plain".to_owned()),
        host_actions: vec![HostActionDto::SavePayload(SavePayloadActionDto {
            suggested_file_name: "payload.txt".to_owned(),
            bytes: b"payload".to_vec(),
            content_type: Some("text/plain".to_owned()),
        })],
    };

    assert_versioned_json_round_trip(response);
}

#[test]
fn fixture_harness_loads_noop_fixture_for_every_supported_hook() {
    let harness = WasmFixtureHarness::new(fixture_root());
    let fixtures = harness.load_all_noop_fixtures().unwrap();
    let hooks = fixtures
        .iter()
        .map(NoopHookFixture::hook)
        .collect::<BTreeSet<_>>();

    assert_eq!(fixtures.len(), HookKind::WASM_DISPATCHED.len());
    assert_eq!(
        hooks,
        HookKind::WASM_DISPATCHED
            .into_iter()
            .collect::<BTreeSet<_>>()
    );

    for hook in HookKind::WASM_DISPATCHED {
        assert!(harness.noop_fixture_path(hook).exists());
    }
}

fn assert_versioned_json_round_trip<T>(value: T)
where
    T: Clone + Debug + PartialEq + Serialize + DeserializeOwned + VersionedDto,
{
    let text = serde_json::to_string(&value).unwrap();
    let parsed = serde_json::from_str::<T>(&text).unwrap();

    assert_eq!(parsed, value);
    assert_eq!(parsed.abi_version(), ABI_VERSION);
}

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/noop")
}
