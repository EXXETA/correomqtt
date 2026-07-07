use serde::Deserialize;
use serde_json::{json, Value};
use uppsala::{parse, XsdValidator};

pub const CONFIGURATION_DESCRIPTION: &str = "Validates XML payloads against the selected XSD schema and records validation errors on matching messages.";

#[cfg(target_arch = "wasm32")]
mod wasm_abi;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XmlXsdValidation {
    Valid,
    Invalid { message: String },
}

pub fn validate_xml_xsd(config: Value, payload: &[u8]) -> XmlXsdValidation {
    let schema_text = match XmlXsdValidatorConfig::schema_text_from_value(config) {
        Ok(schema_text) => schema_text,
        Err(message) => return XmlXsdValidation::Invalid { message },
    };
    validate_payload(&schema_text, payload)
}

pub fn config_schema_document() -> Value {
    json!({
        "type": "object",
        "anyOf": [
            { "required": ["schema_text"] },
            { "required": ["schema_source"] },
            { "required": ["schema"] }
        ],
        "properties": {
            "schema_text": { "type": "string" },
            "schema": { "type": "string" },
            "schema_source": {
                "type": "object",
                "required": ["kind", "text"],
                "properties": {
                    "kind": { "const": "inline" },
                    "text": { "type": "string" }
                },
                "additionalProperties": false
            }
        },
        "additionalProperties": false
    })
}

fn validate_payload(schema_text: &str, payload: &[u8]) -> XmlXsdValidation {
    let payload = match std::str::from_utf8(payload) {
        Ok(payload) => payload,
        Err(_) => return invalid("XML payload must be UTF-8 text."),
    };
    let schema_doc = match parse(schema_text) {
        Ok(schema_doc) => schema_doc,
        Err(source) => return invalid(format!("XSD schema is not valid XML: {source}")),
    };
    let validator = match XsdValidator::from_schema(&schema_doc) {
        Ok(validator) => validator,
        Err(source) => return invalid(format!("XSD schema is invalid: {source}")),
    };
    let payload_doc = match parse(payload) {
        Ok(payload_doc) => payload_doc,
        Err(source) => return invalid(format!("Payload is not valid XML: {source}")),
    };

    let errors = validator.validate(&payload_doc);
    if errors.is_empty() {
        XmlXsdValidation::Valid
    } else {
        invalid(format_validation_errors(&errors))
    }
}

fn format_validation_errors(errors: &[uppsala::ValidationError]) -> String {
    let first = errors
        .first()
        .map(ToString::to_string)
        .unwrap_or_else(|| "XML payload does not match the configured XSD schema.".to_owned());
    if errors.len() == 1 {
        first
    } else {
        format!(
            "{first} (and {} more validation error(s))",
            errors.len() - 1
        )
    }
}

fn invalid(message: impl Into<String>) -> XmlXsdValidation {
    XmlXsdValidation::Invalid {
        message: message.into(),
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
#[derive(Default)]
struct XmlXsdValidatorConfig {
    schema_text: Option<String>,
    schema_source: Option<SchemaSource>,
    schema: Option<String>,
}

impl XmlXsdValidatorConfig {
    fn schema_text_from_value(value: Value) -> Result<String, String> {
        let config = serde_json::from_value::<Self>(value)
            .map_err(|source| format!("XML/XSD validator config is invalid: {source}"))?;
        config.schema_text()
    }

    fn schema_text(self) -> Result<String, String> {
        if let Some(schema_text) = non_empty(self.schema_text) {
            return Ok(schema_text);
        }
        if let Some(SchemaSource::Inline { text }) = self.schema_source {
            if let Some(text) = non_empty(Some(text)) {
                return Ok(text);
            }
        }
        if let Some(schema) = non_empty(self.schema) {
            if schema.trim_start().starts_with('<') {
                return Ok(schema);
            }
            return Err(
                "Legacy XSD schema file paths are not supported; provide inline schema_text."
                    .to_owned(),
            );
        }
        Err("Inline XSD schema text is required.".to_owned())
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum SchemaSource {
    Inline { text: String },
}

fn non_empty(value: Option<String>) -> Option<String> {
    value.filter(|value| !value.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOTE_XSD: &str = r#"
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
"#;

    #[test]
    fn validates_xml_against_inline_schema_text() {
        let validation = validate_xml_xsd(
            json!({ "schema_text": NOTE_XSD }),
            br#"<note><to>Tove</to><from>Jani</from><heading>Reminder</heading><body>Ok</body></note>"#,
        );

        assert_eq!(validation, XmlXsdValidation::Valid);
    }

    #[test]
    fn rejects_xml_that_does_not_match_schema() {
        let validation = validate_xml_xsd(
            json!({ "schema_source": { "kind": "inline", "text": NOTE_XSD } }),
            br#"<note><to>Tove</to><from>Jani</from></note>"#,
        );

        assert!(matches!(
            validation,
            XmlXsdValidation::Invalid { message } if message.contains("Expected")
        ));
    }

    #[test]
    fn rejects_legacy_schema_file_names() {
        let validation = validate_xml_xsd(
            json!({ "schema": "example.xsd" }),
            br#"<note><to>Tove</to></note>"#,
        );

        assert_eq!(
            validation,
            XmlXsdValidation::Invalid {
                message:
                    "Legacy XSD schema file paths are not supported; provide inline schema_text."
                        .to_owned()
            }
        );
    }
}
