use correo_plugins::{
    bundled_plugin_by_id, bundled_plugin_manifests, legacy_plugin_replacement_decisions,
    DetailByteTransformRequest, DetailFormatDto, DetailFormatterRequest, HookInvocation,
    HookOutput, MessageDto, MessageTransformOutcomeDto, MessageValidatorRequest, PluginManifest,
    ValidationResultDto,
};
use serde_json::json;
use std::collections::BTreeSet;

#[test]
fn bundled_manifests_cover_mvp_replacements_with_config_schemas() {
    let manifests = bundled_plugin_manifests();
    let ids = manifests
        .iter()
        .map(|manifest| manifest.id.as_str())
        .collect::<BTreeSet<_>>();

    assert_eq!(
        ids,
        BTreeSet::from([
            "org.correomqtt.plugins.advanced-validator",
            "org.correomqtt.plugins.base64",
            "org.correomqtt.plugins.contains-string-validator",
            "org.correomqtt.plugins.json-format",
            "org.correomqtt.plugins.system-topic",
            "org.correomqtt.plugins.xml-format",
            "org.correomqtt.plugins.xml-xsd-validator",
            "org.correomqtt.plugins.zip-manipulator",
        ])
    );

    for manifest in manifests {
        manifest.validate().unwrap();
        assert!(manifest.config_schema.is_some());
        assert!(!manifest
            .capabilities
            .grants_host_surface(correo_plugins::HostSurface::Filesystem));
        assert!(!manifest
            .capabilities
            .grants_host_surface(correo_plugins::HostSurface::Network));
        assert!(!manifest
            .capabilities
            .grants_host_surface(correo_plugins::HostSurface::Secrets));
        // The system-topic plugin opens live $SYS metric windows and is the
        // only bundled plugin allowed on the MQTT host surface.
        let mqtt_allowed = manifest.id == "org.correomqtt.plugins.system-topic";
        assert_eq!(
            manifest
                .capabilities
                .grants_host_surface(correo_plugins::HostSurface::Mqtt),
            mqtt_allowed
        );
    }
}

#[test]
fn legacy_plugin_decisions_are_explicit_for_supported_and_deferred_plugins() {
    let decisions = legacy_plugin_replacement_decisions();
    let supported = decisions
        .iter()
        .filter(|decision| {
            decision.status == correo_plugins::LegacyPluginReplacementStatus::Supported
        })
        .map(|decision| decision.legacy_plugin_id)
        .collect::<BTreeSet<_>>();
    let unsupported = decisions
        .iter()
        .filter(|decision| {
            decision.status == correo_plugins::LegacyPluginReplacementStatus::Unsupported
        })
        .map(|decision| decision.legacy_plugin_id)
        .collect::<BTreeSet<_>>();

    assert_eq!(
        supported,
        BTreeSet::from([
            "advanced-validator",
            "base64",
            "contains-string-validator",
            "json-format",
            "save-manipulator",
            "systopic",
            "xml-format",
            "xml-xsd-validator",
            "zip-manipulator",
        ])
    );
    assert!(unsupported.is_empty());
    assert!(decisions
        .iter()
        .filter(|decision| {
            decision.status == correo_plugins::LegacyPluginReplacementStatus::Unsupported
        })
        .all(|decision| !decision.reason.is_empty() && decision.replacement_plugin_id.is_none()));
}

#[test]
fn xml_xsd_package_manifest_declares_validator_hook_without_host_access() {
    let manifest = PluginManifest::from_toml_str(include_str!(
        "../../../plugins/correo-plugins-xml-xsd-validator/plugin.toml"
    ))
    .unwrap();

    manifest.validate().unwrap();
    assert_eq!(manifest.id, "org.correomqtt.plugins.xml-xsd-validator");
    assert_eq!(
        manifest.capabilities.hooks,
        vec![correo_plugins::HookKind::MessageValidator]
    );
    assert!(!manifest
        .capabilities
        .grants_host_surface(correo_plugins::HostSurface::Filesystem));
    assert!(!manifest
        .capabilities
        .grants_host_surface(correo_plugins::HostSurface::Network));
    assert!(!manifest
        .capabilities
        .grants_host_surface(correo_plugins::HostSurface::Secrets));
    assert!(!manifest
        .capabilities
        .grants_host_surface(correo_plugins::HostSurface::Mqtt));
}

#[test]
fn base64_replacement_encodes_outgoing_and_decodes_incoming() {
    let plugin = bundled_plugin_by_id("org.correomqtt.plugins.base64").unwrap();
    let outgoing = plugin
        .dispatch(HookInvocation::OutgoingMessageTransform(
            correo_plugins::OutgoingMessageTransformRequest::new(MessageDto::new(
                "demo/topic",
                b"hello".to_vec(),
            )),
        ))
        .unwrap();

    assert_transformed_payload(outgoing, b"aGVsbG8=");

    let incoming = plugin
        .dispatch(HookInvocation::IncomingMessageTransform(
            correo_plugins::IncomingMessageTransformRequest::new(MessageDto::new(
                "demo/topic",
                b"aGVsbG8=".to_vec(),
            )),
        ))
        .unwrap();

    match incoming {
        HookOutput::IncomingMessageTransform(response) => match response.outcome {
            MessageTransformOutcomeDto::Replace { message } => {
                assert_eq!(message.payload, b"hello");
            }
            other => panic!("unexpected outcome: {other:?}"),
        },
        other => panic!("unexpected output: {other:?}"),
    }
}

#[test]
fn detail_formatters_return_pretty_json_and_xml_text() {
    let json = bundled_plugin_by_id("org.correomqtt.plugins.json-format").unwrap();
    let output = json
        .dispatch(HookInvocation::DetailFormatter(
            DetailFormatterRequest::new(br#"{"ok":true,"items":[1,2]}"#.to_vec()),
        ))
        .unwrap();
    assert_formatted(output, DetailFormatDto::Json, "\"items\": [");

    let xml = bundled_plugin_by_id("org.correomqtt.plugins.xml-format").unwrap();
    let output = xml
        .dispatch(HookInvocation::DetailFormatter(
            DetailFormatterRequest::new(br#"<root><item id="1">ok</item></root>"#.to_vec()),
        ))
        .unwrap();
    assert_formatted(output, DetailFormatDto::Xml, "  <item id=\"1\">");
}

#[test]
fn contains_string_validator_supports_case_sensitive_config() {
    let plugin = bundled_plugin_by_id("org.correomqtt.plugins.contains-string-validator").unwrap();
    let mut request =
        MessageValidatorRequest::new(MessageDto::new("demo/topic", b"Telemetry READY".to_vec()));
    request.config = json!({
        "text": "ready",
        "case_sensitive": false
    });

    let output = plugin
        .dispatch(HookInvocation::MessageValidator(request))
        .unwrap();
    match output {
        HookOutput::MessageValidator(response) => {
            assert_eq!(response.result, ValidationResultDto::Valid);
        }
        other => panic!("unexpected output: {other:?}"),
    }
}

#[test]
fn zip_manipulator_transforms_detail_bytes_with_expansion_limits() {
    let plugin = bundled_plugin_by_id("org.correomqtt.plugins.zip-manipulator").unwrap();
    let mut request = DetailByteTransformRequest::new(b"payload".to_vec());
    request.config = json!({ "operation": "zip" });

    let output = plugin
        .dispatch(HookInvocation::DetailByteTransform(request))
        .unwrap();
    let (zipped, content_type) = assert_detail_bytes(output);

    assert!(zipped.starts_with(&[0x1f, 0x8b]));
    assert_eq!(content_type.as_deref(), Some("application/gzip"));

    let mut request = DetailByteTransformRequest::new(zipped.clone());
    request.content_type = content_type;
    request.config = json!({
        "operation": "unzip",
        "max_output_bytes": 128
    });
    let output = plugin
        .dispatch(HookInvocation::DetailByteTransform(request))
        .unwrap();
    let (unzipped, content_type) = assert_detail_bytes(output);

    assert_eq!(unzipped, b"payload");
    assert_eq!(content_type, None);

    let mut request = DetailByteTransformRequest::new(zipped);
    request.config = json!({
        "operation": "unzip",
        "max_output_bytes": 3
    });
    let error = plugin
        .dispatch(HookInvocation::DetailByteTransform(request))
        .unwrap_err();
    assert!(error.to_string().contains("output exceeded 3 bytes"));
}

#[test]
fn xml_xsd_validator_accepts_payload_that_matches_inline_schema() {
    let plugin = bundled_plugin_by_id("org.correomqtt.plugins.xml-xsd-validator").unwrap();
    let mut request = MessageValidatorRequest::new(MessageDto::new(
        "demo/topic",
        br#"<note><to>Tove</to><from>Jani</from><heading>Reminder</heading><body>Ok</body></note>"#
            .to_vec(),
    ));
    request.config = json!({ "schema_text": note_xsd() });

    let output = plugin
        .dispatch(HookInvocation::MessageValidator(request))
        .unwrap();

    match output {
        HookOutput::MessageValidator(response) => {
            assert_eq!(response.result, ValidationResultDto::Valid);
        }
        other => panic!("unexpected output: {other:?}"),
    }
}

#[test]
fn xml_xsd_validator_rejects_payload_that_misses_required_schema_elements() {
    let plugin = bundled_plugin_by_id("org.correomqtt.plugins.xml-xsd-validator").unwrap();
    let mut request = MessageValidatorRequest::new(MessageDto::new(
        "demo/topic",
        br#"<note><to>Tove</to><from>Jani</from></note>"#.to_vec(),
    ));
    request.config = json!({
        "schema_source": {
            "kind": "inline",
            "text": note_xsd()
        }
    });

    let output = plugin
        .dispatch(HookInvocation::MessageValidator(request))
        .unwrap();

    match output {
        HookOutput::MessageValidator(response) => {
            assert!(matches!(
                response.result,
                ValidationResultDto::Invalid { message } if message.contains("Expected")
            ));
        }
        other => panic!("unexpected output: {other:?}"),
    }
}

#[test]
fn xml_xsd_validator_rejects_legacy_file_schema_config() {
    let plugin = bundled_plugin_by_id("org.correomqtt.plugins.xml-xsd-validator").unwrap();
    let mut request = MessageValidatorRequest::new(MessageDto::new(
        "demo/topic",
        br#"<note><to>Tove</to></note>"#.to_vec(),
    ));
    request.config = json!({ "schema": "example.xsd" });

    let output = plugin
        .dispatch(HookInvocation::MessageValidator(request))
        .unwrap();

    match output {
        HookOutput::MessageValidator(response) => {
            assert_eq!(
                response.result,
                ValidationResultDto::Invalid {
                    message:
                        "Legacy XSD schema file paths are not supported; provide inline schema_text."
                            .to_owned()
                }
            );
        }
        other => panic!("unexpected output: {other:?}"),
    }
}

fn assert_transformed_payload(output: HookOutput, expected: &[u8]) {
    match output {
        HookOutput::OutgoingMessageTransform(response) => match response.outcome {
            MessageTransformOutcomeDto::Replace { message } => {
                assert_eq!(message.payload, expected);
            }
            other => panic!("unexpected outcome: {other:?}"),
        },
        other => panic!("unexpected output: {other:?}"),
    }
}

fn assert_detail_bytes(output: HookOutput) -> (Vec<u8>, Option<String>) {
    match output {
        HookOutput::DetailByteTransform(response) => (response.bytes, response.content_type),
        other => panic!("unexpected output: {other:?}"),
    }
}

fn note_xsd() -> &'static str {
    r#"
<xs:schema elementFormDefault="qualified" xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:element name="note">
    <xs:complexType>
      <xs:sequence>
        <xs:element type="xs:string" name="to"/>
        <xs:element type="xs:string" name="from"/>
        <xs:element type="xs:string" name="heading"/>
        <xs:element type="xs:string" name="body"/>
      </xs:sequence>
    </xs:complexType>
  </xs:element>
</xs:schema>
"#
}

fn assert_formatted(output: HookOutput, format: DetailFormatDto, expected_text: &str) {
    match output {
        HookOutput::DetailFormatter(response) => {
            assert_eq!(response.output.format, format);
            assert!(response.output.text.contains(expected_text));
            assert!(response.output.diagnostics.is_empty());
        }
        other => panic!("unexpected output: {other:?}"),
    }
}
