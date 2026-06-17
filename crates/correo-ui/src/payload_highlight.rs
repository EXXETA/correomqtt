use std::sync::Arc;

use correo_core::{PayloadSyntaxKind, PayloadSyntaxSpan};
use egui::{text::LayoutJob, Color32, FontId, TextFormat, TextStyle, Ui};

use crate::PayloadHighlighter;

#[derive(Clone, Copy)]
struct Palette {
    plain: Color32,
    key: Color32,
    string: Color32,
    number: Color32,
    keyword: Color32,
    punct: Color32,
    tag: Color32,
    attr: Color32,
    comment: Color32,
    method: Color32,
    class: Color32,
}

pub(crate) fn layouter(
    highlighter: Option<PayloadHighlighter>,
    active_plugin_ids: Vec<String>,
) -> impl FnMut(&Ui, &str, f32) -> Arc<egui::Galley> {
    move |ui, text, wrap_width| {
        let mut job = highlight_payload(ui, text, highlighter.as_ref(), &active_plugin_ids);
        job.wrap.max_width = wrap_width;
        ui.fonts(|fonts| fonts.layout_job(job))
    }
}

pub(crate) fn javascript_layouter() -> impl FnMut(&Ui, &str, f32) -> Arc<egui::Galley> {
    move |ui, text, wrap_width| {
        let font = TextStyle::Monospace.resolve(ui.style());
        let mut job = highlight_javascript(text, font, palette(ui));
        job.wrap.max_width = wrap_width;
        ui.fonts(|fonts| fonts.layout_job(job))
    }
}

fn highlight_payload(
    ui: &Ui,
    text: &str,
    highlighter: Option<&PayloadHighlighter>,
    active_plugin_ids: &[String],
) -> LayoutJob {
    let font = TextStyle::Monospace.resolve(ui.style());
    let palette = palette(ui);
    if let Some(spans) = highlighter.and_then(|highlight| highlight(text, active_plugin_ids)) {
        highlight_spans(text, spans, font, palette)
    } else {
        plain_job(text, font, palette.plain)
    }
}

fn palette(ui: &Ui) -> Palette {
    let visuals = ui.visuals();
    if visuals.dark_mode {
        Palette {
            plain: visuals.text_color(),
            key: Color32::from_rgb(156, 220, 254),
            string: Color32::from_rgb(206, 145, 120),
            number: Color32::from_rgb(181, 206, 168),
            keyword: Color32::from_rgb(86, 156, 214),
            punct: visuals.weak_text_color(),
            tag: Color32::from_rgb(86, 156, 214),
            attr: Color32::from_rgb(156, 220, 254),
            comment: Color32::from_rgb(106, 153, 85),
            method: Color32::from_rgb(220, 220, 170),
            class: Color32::from_rgb(78, 201, 176),
        }
    } else {
        Palette {
            plain: visuals.text_color(),
            key: Color32::from_rgb(0, 92, 160),
            string: Color32::from_rgb(163, 21, 21),
            number: Color32::from_rgb(9, 134, 88),
            keyword: Color32::from_rgb(0, 0, 255),
            punct: visuals.weak_text_color(),
            tag: Color32::from_rgb(128, 0, 0),
            attr: Color32::from_rgb(255, 0, 0),
            comment: Color32::from_rgb(0, 128, 0),
            method: Color32::from_rgb(121, 94, 38),
            class: Color32::from_rgb(38, 127, 153),
        }
    }
}

fn plain_job(text: &str, font: FontId, color: Color32) -> LayoutJob {
    let mut job = LayoutJob::default();
    append(&mut job, text, font, color);
    job
}

fn highlight_spans(
    text: &str,
    mut spans: Vec<PayloadSyntaxSpan>,
    font: FontId,
    palette: Palette,
) -> LayoutJob {
    let mut job = LayoutJob::default();
    spans.sort_by_key(|span| span.start);
    let mut index = 0;
    for span in spans {
        if span.start < index || span.start >= span.end || span.end > text.len() {
            continue;
        }
        if index < span.start {
            append(
                &mut job,
                &text[index..span.start],
                font.clone(),
                palette.plain,
            );
        }
        append(
            &mut job,
            &text[span.start..span.end],
            font.clone(),
            syntax_color(span.kind, palette),
        );
        index = span.end;
    }
    if index < text.len() {
        append(&mut job, &text[index..], font, palette.plain);
    }
    job
}

fn syntax_color(kind: PayloadSyntaxKind, palette: Palette) -> Color32 {
    match kind {
        PayloadSyntaxKind::Key => palette.key,
        PayloadSyntaxKind::String => palette.string,
        PayloadSyntaxKind::Number => palette.number,
        PayloadSyntaxKind::Keyword => palette.keyword,
        PayloadSyntaxKind::Punctuation => palette.punct,
        PayloadSyntaxKind::Tag => palette.tag,
        PayloadSyntaxKind::Attribute => palette.attr,
        PayloadSyntaxKind::Comment => palette.comment,
    }
}

fn highlight_javascript(text: &str, font: FontId, palette: Palette) -> LayoutJob {
    let mut job = LayoutJob::default();
    let mut index = 0;
    while index < text.len() {
        let ch = text[index..].chars().next().unwrap_or_default();
        if text[index..].starts_with("//") {
            let end = text[index..]
                .find('\n')
                .map_or(text.len(), |offset| index + offset);
            append(&mut job, &text[index..end], font.clone(), palette.comment);
            index = end;
        } else if text[index..].starts_with("/*") {
            let end = text[index..]
                .find("*/")
                .map_or(text.len(), |offset| index + offset + 2);
            append(&mut job, &text[index..end], font.clone(), palette.comment);
            index = end;
        } else if matches!(ch, '"' | '\'' | '`') {
            let end = quoted_end_escaped(text, index, ch);
            append(&mut job, &text[index..end], font.clone(), palette.string);
            index = end;
        } else if ch.is_ascii_digit() {
            let end = take_while(text, index, |c| {
                c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '+' | '-')
            });
            append(&mut job, &text[index..end], font.clone(), palette.number);
            index = end;
        } else if is_js_ident_start(ch) {
            let end = take_while(text, index, is_js_ident_continue);
            let word = &text[index..end];
            let color = if is_js_keyword(word) {
                palette.keyword
            } else if is_class_like(word) || previous_word_is_new(text, index) {
                palette.class
            } else if previous_non_ws(text.as_bytes(), index) == Some(b'.')
                || next_non_ws(text.as_bytes(), end) == Some(b'(')
            {
                palette.method
            } else {
                palette.plain
            };
            append(&mut job, word, font.clone(), color);
            index = end;
        } else if is_js_punctuation(ch) {
            append(
                &mut job,
                &text[index..index + ch.len_utf8()],
                font.clone(),
                palette.punct,
            );
            index += ch.len_utf8();
        } else {
            append(
                &mut job,
                &text[index..index + ch.len_utf8()],
                font.clone(),
                palette.plain,
            );
            index += ch.len_utf8();
        }
    }
    job
}

fn append(job: &mut LayoutJob, text: &str, font: FontId, color: Color32) {
    job.append(text, 0.0, TextFormat::simple(font, color));
}

fn quoted_end_escaped(text: &str, start: usize, quote: char) -> usize {
    let mut escaped = false;
    for (offset, ch) in text[start + 1..].char_indices() {
        if escaped {
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == quote {
            return start + 1 + offset + ch.len_utf8();
        }
    }
    text.len()
}

fn take_while(text: &str, start: usize, mut predicate: impl FnMut(char) -> bool) -> usize {
    for (offset, ch) in text[start..].char_indices() {
        if !predicate(ch) {
            return start + offset;
        }
    }
    text.len()
}

fn next_non_ws(bytes: &[u8], start: usize) -> Option<u8> {
    bytes
        .get(start..)?
        .iter()
        .copied()
        .find(|byte| !byte.is_ascii_whitespace())
}

fn previous_non_ws(bytes: &[u8], start: usize) -> Option<u8> {
    bytes
        .get(..start)?
        .iter()
        .rev()
        .copied()
        .find(|byte| !byte.is_ascii_whitespace())
}

fn is_js_ident_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || matches!(ch, '_' | '$')
}

fn is_js_ident_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '$')
}

fn is_js_punctuation(ch: char) -> bool {
    matches!(
        ch,
        '{' | '}'
            | '['
            | ']'
            | '('
            | ')'
            | ';'
            | ','
            | '.'
            | ':'
            | '?'
            | '!'
            | '+'
            | '-'
            | '*'
            | '/'
            | '%'
            | '='
            | '<'
            | '>'
            | '&'
            | '|'
    )
}

fn is_js_keyword(word: &str) -> bool {
    matches!(
        word,
        "async"
            | "await"
            | "break"
            | "case"
            | "catch"
            | "class"
            | "const"
            | "continue"
            | "debugger"
            | "default"
            | "delete"
            | "do"
            | "else"
            | "export"
            | "extends"
            | "false"
            | "finally"
            | "for"
            | "from"
            | "function"
            | "if"
            | "import"
            | "in"
            | "instanceof"
            | "let"
            | "new"
            | "null"
            | "of"
            | "return"
            | "static"
            | "super"
            | "switch"
            | "this"
            | "throw"
            | "true"
            | "try"
            | "typeof"
            | "undefined"
            | "var"
            | "void"
            | "while"
            | "yield"
    )
}

fn is_class_like(word: &str) -> bool {
    word.chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_uppercase())
}

fn previous_word_is_new(text: &str, index: usize) -> bool {
    let Some(prefix) = text.get(..index) else {
        return false;
    };
    let trimmed = prefix.trim_end();
    let Some(end) = trimmed
        .char_indices()
        .last()
        .map(|(index, ch)| index + ch.len_utf8())
    else {
        return false;
    };
    let start = trimmed[..end]
        .char_indices()
        .rev()
        .find(|(_, ch)| !is_js_ident_continue(*ch))
        .map_or(0, |(index, ch)| index + ch.len_utf8());
    &trimmed[start..end] == "new"
}
