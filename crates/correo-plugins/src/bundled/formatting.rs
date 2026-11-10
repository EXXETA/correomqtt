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
    Error,
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
    let spans = highlight_json_syntax(text)?.into_iter().map(json_span);
    let Some(error) = json_error_range(text) else {
        return Some(spans.collect());
    };
    let mut marked = Vec::new();
    for span in spans {
        if span.end <= error.start || span.start >= error.end {
            marked.push(span);
            continue;
        }
        if span.start < error.start {
            marked.push(PayloadSyntaxSpan {
                end: error.start,
                ..span
            });
        }
        if span.end > error.end {
            marked.push(PayloadSyntaxSpan {
                start: error.end,
                ..span
            });
        }
    }
    marked.push(PayloadSyntaxSpan {
        start: error.start,
        end: error.end,
        kind: PayloadSyntaxKind::Error,
    });
    Some(marked)
}

/// Rewrites only the whitespace between tokens, so key order and number spelling stay as typed.
pub fn reformat_json(text: &str, minify: bool) -> Option<String> {
    serde_json::from_str::<serde::de::IgnoredAny>(text).ok()?;
    let mut output = String::with_capacity(text.len());
    let mut depth = 0_usize;
    let mut chars = text.chars().peekable();
    let mut in_string = false;
    while let Some(ch) = chars.next() {
        if in_string {
            output.push(ch);
            if ch == '\\' {
                output.extend(chars.next());
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        if ch.is_whitespace() {
            continue;
        }
        if minify {
            in_string = ch == '"';
            output.push(ch);
            continue;
        }
        match ch {
            '{' | '[' => {
                output.push(ch);
                while chars.next_if(|next| next.is_whitespace()).is_some() {}
                if let Some(closer) = chars.next_if(|next| matches!(next, '}' | ']')) {
                    output.push(closer);
                } else {
                    depth += 1;
                    push_line(&mut output, depth);
                }
            }
            '}' | ']' => {
                depth = depth.saturating_sub(1);
                push_line(&mut output, depth);
                output.push(ch);
            }
            ',' => {
                output.push(ch);
                push_line(&mut output, depth);
            }
            ':' => output.push_str(": "),
            _ => {
                in_string = ch == '"';
                output.push(ch);
            }
        }
    }
    Some(output)
}

/// Fixes the common hand-typing mistakes and returns the text only if it is valid JSON afterwards.
pub fn repair_json(text: &str) -> Option<String> {
    let mut output = String::with_capacity(text.len() + 8);
    let mut rest = text;
    while let Some(ch) = rest.chars().next() {
        let consumed = match ch {
            '"' => {
                let end = quoted_end(rest, '"');
                output.push_str(&rest[..end]);
                end
            }
            '\'' => {
                let end = quoted_end(rest, '\'');
                let inner = rest[1..end].strip_suffix('\'').unwrap_or(&rest[1..end]);
                output.push('"');
                output.push_str(&inner.replace("\\'", "'").replace('"', "\\\""));
                output.push('"');
                end
            }
            ',' if rest[1..].trim_start().starts_with(['}', ']']) => 1,
            _ if ch.is_whitespace() || "{}[]:,".contains(ch) => {
                output.push(ch);
                ch.len_utf8()
            }
            _ => {
                let end = rest
                    .find(|next| "{}[]:,\"'\n".contains(next))
                    .unwrap_or(rest.len());
                let word = rest[..end].trim_end();
                if is_valid_json(word) {
                    output.push_str(word);
                } else {
                    output.push_str(&serde_json::to_string(word).ok()?);
                }
                word.len()
            }
        };
        rest = &rest[consumed..];
    }
    is_valid_json(&output).then_some(output)
}

fn quoted_end(text: &str, quote: char) -> usize {
    let mut escaped = false;
    for (index, ch) in text.char_indices().skip(1) {
        if escaped {
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == quote {
            return index + 1;
        }
    }
    text.len()
}

fn is_valid_json(text: &str) -> bool {
    serde_json::from_str::<serde::de::IgnoredAny>(text).is_ok()
}

fn push_line(output: &mut String, depth: usize) {
    output.push('\n');
    output.push_str(&"  ".repeat(depth));
}

fn json_error_range(text: &str) -> Option<std::ops::Range<usize>> {
    let error = serde_json::from_str::<serde::de::IgnoredAny>(text).err()?;
    let line_start: usize = text
        .split_inclusive('\n')
        .take(error.line().saturating_sub(1))
        .map(str::len)
        .sum();
    // serde_json counts the offending byte as already read, so it sits one before the column.
    let reported = (line_start + error.column().saturating_sub(1)).min(text.len());
    // At the end of input or on whitespace there is nothing to mark, so mark the last token.
    let (start, marked) = text
        .char_indices()
        .take_while(|(index, _)| *index <= reported)
        .filter(|(_, ch)| !ch.is_whitespace())
        .last()?;
    Some(start..start + marked.len_utf8())
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

#[cfg(test)]
mod tests {
    use super::*;

    fn error_text(text: &str) -> Option<&str> {
        highlight_json(text)?
            .into_iter()
            .find(|span| span.kind == PayloadSyntaxKind::Error)
            .map(|span| &text[span.start..span.end])
    }

    #[test]
    fn valid_json_has_no_error_span() {
        assert_eq!(error_text("{\"a\": [1, true]}"), None);
    }

    #[test]
    fn marks_the_character_the_parser_rejects() {
        assert_eq!(error_text("{\n  \"ä\": 1,\n  \"b\": x\n}"), Some("x"));
    }

    #[test]
    fn marks_the_last_token_when_input_ends_early() {
        assert_eq!(error_text("{\"a\": [1,\n"), Some(","));
    }

    #[test]
    fn format_keeps_key_order_and_number_spelling() {
        assert_eq!(
            reformat_json("{\"b\":1.50,\"a\":[ ],\"c\":[1,{\"d\":\"x, \\\"y\\\": {\"}]}", false).as_deref(),
            Some("{\n  \"b\": 1.50,\n  \"a\": [],\n  \"c\": [\n    1,\n    {\n      \"d\": \"x, \\\"y\\\": {\"\n    }\n  ]\n}")
        );
    }

    #[test]
    fn minify_removes_whitespace_outside_strings() {
        assert_eq!(
            reformat_json("{\n  \"a b\": [1, 2],\n  \"c\": \"d  e\"\n}\n", true).as_deref(),
            Some("{\"a b\":[1,2],\"c\":\"d  e\"}")
        );
    }

    #[test]
    fn invalid_json_is_not_reformatted() {
        assert_eq!(reformat_json("{\"a\": }", false), None);
    }

    #[test]
    fn repair_quotes_bare_words_and_drops_trailing_commas() {
        assert_eq!(
            repair_json("{name: 'it\\'s \"x\"', on: true, n: -1.5e3, note: hello world,\n}").as_deref(),
            Some("{\"name\": \"it's \\\"x\\\"\", \"on\": true, \"n\": -1.5e3, \"note\": \"hello world\"\n}")
        );
    }

    #[test]
    fn repair_leaves_quoted_strings_untouched() {
        assert_eq!(
            repair_json("{\"a, b\": \"it's: {x}\", c: [1,],}").as_deref(),
            Some("{\"a, b\": \"it's: {x}\", \"c\": [1]}")
        );
    }

    #[test]
    fn repair_gives_up_when_the_result_is_still_invalid() {
        assert_eq!(repair_json("{\"a\": [1, 2"), None);
    }

    #[test]
    fn error_span_does_not_overlap_syntax_spans() {
        let mut spans = highlight_json("{\"a\": \"b\" \"c\"}").expect("json spans");
        spans.sort_by_key(|span| span.start);
        assert!(spans.windows(2).all(|pair| pair[0].end <= pair[1].start));
        assert!(spans
            .iter()
            .any(|span| span.kind == PayloadSyntaxKind::Error));
    }
}
