fn empty_config_schema() -> ConfigSchemaMetadata {
    ConfigSchemaMetadata {
        schema_version: 1,
        document: json!({
            "type": "object",
            "additionalProperties": false
        }),
    }
}

fn contains_string_config_schema() -> ConfigSchemaMetadata {
    ConfigSchemaMetadata {
        schema_version: 1,
        document: json!({
            "type": "object",
            "required": ["text"],
            "properties": {
                "text": { "type": "string" },
                "case_sensitive": { "type": "boolean", "default": true }
            },
            "additionalProperties": false
        }),
    }
}

fn xml_xsd_config_schema() -> ConfigSchemaMetadata {
    ConfigSchemaMetadata {
        schema_version: 1,
        document: config_schema_document(),
    }
}

fn bundled_export_name(hook: HookKind) -> &'static str {
    match hook {
        HookKind::OutgoingMessageTransform => "builtin_outgoing_message_transform",
        HookKind::IncomingMessageTransform => "builtin_incoming_message_transform",
        HookKind::MessageValidator => "builtin_message_validator",
        HookKind::DetailByteTransform => "builtin_detail_byte_transform",
        HookKind::DetailFormatter => "builtin_detail_formatter",
        HookKind::PayloadHighlighter => "builtin_payload_highlighter",
    }
}

fn supported_builtin(
    legacy_plugin_id: &'static str,
    replacement_plugin_id: &'static str,
) -> LegacyPluginReplacementDecision {
    supported(
        legacy_plugin_id,
        replacement_plugin_id,
        "Covered by a bundled Rust replacement for the MVP hook surface.",
    )
}

fn supported(
    legacy_plugin_id: &'static str,
    replacement_plugin_id: &'static str,
    reason: &'static str,
) -> LegacyPluginReplacementDecision {
    LegacyPluginReplacementDecision {
        legacy_plugin_id,
        status: LegacyPluginReplacementStatus::Supported,
        replacement_plugin_id: Some(replacement_plugin_id),
        reason,
    }
}
