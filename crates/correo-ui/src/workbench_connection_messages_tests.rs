fn auto_scroll_id(origin: MessageOrigin) -> Id {
    match origin {
        MessageOrigin::Outgoing => Id::new("outgoing-messages-auto-scroll"),
        MessageOrigin::Incoming => Id::new("incoming-messages-auto-scroll"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xml_validator_status_comes_from_diagnostics_not_workflow_config() {
        assert_eq!(validation_status_for_row(&[]), None);

        let diagnostics = vec![correo_core::MessageDiagnosticRow {
            severity: correo_core::PluginDiagnosticSeverity::Info,
            hook: Some(correo_core::PluginHookKind::Validator),
            plugin_id: Some("org.correomqtt.plugins.xml-xsd-validator".to_owned()),
            message: "XML payload validated".to_owned(),
        }];
        assert_eq!(
            validation_status_for_row(&diagnostics),
            Some(ValidationStatus::Validated)
        );
    }
}
