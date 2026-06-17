use super::BundledPluginError;
use crate::{
    DetailFormatDto, DetailFormatterRequest, DetailFormatterResponse, FormattedDetailDto,
    HookDiagnosticDto, HookDiagnosticSeverityDto, HookKind, ABI_VERSION,
};
use correo_plugin_xml_format::{
    format_xml_bytes, highlight_xml_syntax, XmlDetailFormat, XmlFormatDiagnostic,
    XmlFormatDiagnosticSeverity, XmlFormatOutput, XmlSyntaxKind, XmlSyntaxSpan,
};
use correo_plugins_json_format::{
    format_json_bytes, highlight_json_syntax, JsonDetailFormat, JsonFormatDiagnostic,
    JsonFormatDiagnosticSeverity, JsonFormatOutput, JsonSyntaxKind, JsonSyntaxSpan,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayloadSyntaxKind {
    Key,
    String,
    Number,
    Keyword,
    Punctuation,
    Tag,
    Attribute,
    Comment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PayloadSyntaxSpan {
    pub start: usize,
    pub end: usize,
    pub kind: PayloadSyntaxKind,
}

pub(super) fn format_json(
    request: DetailFormatterRequest,
    plugin_id: &str,
) -> Result<DetailFormatterResponse, BundledPluginError> {
    format_json_bytes(request.bytes)
        .map(json_output)
        .map_err(|source| BundledPluginError::InvalidUtf8 {
            plugin_id: plugin_id.to_owned(),
            hook: HookKind::DetailFormatter,
            source: source.into_source(),
        })
}

pub(super) fn format_xml(
    request: DetailFormatterRequest,
    plugin_id: &str,
) -> Result<DetailFormatterResponse, BundledPluginError> {
    format_xml_bytes(request.bytes)
        .map(xml_output)
        .map_err(|source| BundledPluginError::InvalidUtf8 {
            plugin_id: plugin_id.to_owned(),
            hook: HookKind::DetailFormatter,
            source: source.into_source(),
        })
}

pub fn highlight_json(text: &str) -> Option<Vec<PayloadSyntaxSpan>> {
    highlight_json_syntax(text).map(|spans| spans.into_iter().map(json_span).collect())
}

pub fn highlight_xml(text: &str) -> Option<Vec<PayloadSyntaxSpan>> {
    highlight_xml_syntax(text).map(|spans| spans.into_iter().map(xml_span).collect())
}

fn json_span(span: JsonSyntaxSpan) -> PayloadSyntaxSpan {
    PayloadSyntaxSpan {
        start: span.start,
        end: span.end,
        kind: match span.kind {
            JsonSyntaxKind::Key => PayloadSyntaxKind::Key,
            JsonSyntaxKind::String => PayloadSyntaxKind::String,
            JsonSyntaxKind::Number => PayloadSyntaxKind::Number,
            JsonSyntaxKind::Keyword => PayloadSyntaxKind::Keyword,
            JsonSyntaxKind::Punctuation => PayloadSyntaxKind::Punctuation,
        },
    }
}

fn xml_span(span: XmlSyntaxSpan) -> PayloadSyntaxSpan {
    PayloadSyntaxSpan {
        start: span.start,
        end: span.end,
        kind: match span.kind {
            XmlSyntaxKind::String => PayloadSyntaxKind::String,
            XmlSyntaxKind::Punctuation => PayloadSyntaxKind::Punctuation,
            XmlSyntaxKind::Tag => PayloadSyntaxKind::Tag,
            XmlSyntaxKind::Attribute => PayloadSyntaxKind::Attribute,
            XmlSyntaxKind::Comment => PayloadSyntaxKind::Comment,
        },
    }
}

fn formatted_detail(
    format: DetailFormatDto,
    text: String,
    diagnostics: Vec<HookDiagnosticDto>,
) -> DetailFormatterResponse {
    DetailFormatterResponse {
        abi_version: ABI_VERSION,
        output: FormattedDetailDto {
            format,
            text,
            diagnostics,
        },
    }
}

fn json_output(output: JsonFormatOutput) -> DetailFormatterResponse {
    formatted_detail(
        json_format(output.format),
        output.text,
        output
            .diagnostics
            .into_iter()
            .map(json_diagnostic)
            .collect(),
    )
}

fn json_format(format: JsonDetailFormat) -> DetailFormatDto {
    match format {
        JsonDetailFormat::Json => DetailFormatDto::Json,
        JsonDetailFormat::PlainText => DetailFormatDto::PlainText,
    }
}

fn json_diagnostic(diagnostic: JsonFormatDiagnostic) -> HookDiagnosticDto {
    HookDiagnosticDto {
        severity: match diagnostic.severity {
            JsonFormatDiagnosticSeverity::Warning => HookDiagnosticSeverityDto::Warning,
        },
        message: diagnostic.message,
    }
}

fn xml_output(output: XmlFormatOutput) -> DetailFormatterResponse {
    formatted_detail(
        xml_format(output.format),
        output.text,
        output.diagnostics.into_iter().map(xml_diagnostic).collect(),
    )
}

fn xml_format(format: XmlDetailFormat) -> DetailFormatDto {
    match format {
        XmlDetailFormat::Xml => DetailFormatDto::Xml,
        XmlDetailFormat::PlainText => DetailFormatDto::PlainText,
    }
}

fn xml_diagnostic(diagnostic: XmlFormatDiagnostic) -> HookDiagnosticDto {
    HookDiagnosticDto {
        severity: match diagnostic.severity {
            XmlFormatDiagnosticSeverity::Warning => HookDiagnosticSeverityDto::Warning,
        },
        message: diagnostic.message,
    }
}
