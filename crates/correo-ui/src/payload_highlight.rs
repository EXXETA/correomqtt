use std::{
    collections::{HashMap, HashSet, VecDeque},
    hash::{Hash, Hasher},
    sync::{mpsc, Arc, Mutex},
};

use correo_core::{PayloadSyntaxKind, PayloadSyntaxSpan};
use egui::{text::LayoutJob, Color32, Context, FontId, Id, TextFormat, TextStyle, Ui};

use crate::PayloadHighlighter;
#[path = "payload_highlight_javascript.rs"]
mod javascript;

use javascript::{
    is_class_like, is_js_ident_continue, is_js_ident_start, is_js_keyword, is_js_punctuation,
    previous_word_is_new,
};

const HIGHLIGHT_CACHE_CAPACITY: usize = 32;
const HIGHLIGHT_WORKER_CAPACITY: usize = 4;
const MAX_CACHED_SPANS: usize = 100_000;
const HIGHLIGHT_CACHE_ID: &str = "payload-highlight-cache";

type HighlightResult = Option<Arc<[PayloadSyntaxSpan]>>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct HighlightKey {
    payload_hash: u64,
    payload_len: usize,
    plugin_hash: u64,
}

impl HighlightKey {
    fn new(payload: &str, active_plugin_ids: &[String]) -> Self {
        let mut payload_hasher = std::hash::DefaultHasher::new();
        payload.hash(&mut payload_hasher);
        let mut plugin_hasher = std::hash::DefaultHasher::new();
        active_plugin_ids.hash(&mut plugin_hasher);
        Self {
            payload_hash: payload_hasher.finish(),
            payload_len: payload.len(),
            plugin_hash: plugin_hasher.finish(),
        }
    }
}

struct HighlightCache {
    results: HashMap<HighlightKey, HighlightResult>,
    order: VecDeque<HighlightKey>,
    pending: HashSet<HighlightKey>,
    jobs: mpsc::SyncSender<HighlightJob>,
    receiver: mpsc::Receiver<(HighlightKey, HighlightResult)>,
}

type HighlightJob = (
    HighlightKey,
    String,
    Vec<String>,
    Context,
    PayloadHighlighter,
);

impl Default for HighlightCache {
    fn default() -> Self {
        let (jobs, job_receiver) = mpsc::sync_channel::<HighlightJob>(HIGHLIGHT_WORKER_CAPACITY);
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            while let Ok((key, payload, active_plugin_ids, context, highlighter)) =
                job_receiver.recv()
            {
                let result = highlighter(&payload, &active_plugin_ids).and_then(|mut spans| {
                    if spans.len() > MAX_CACHED_SPANS {
                        return None;
                    }
                    spans.sort_by_key(|span| span.start);
                    Some(Arc::from(spans))
                });
                let _ = sender.send((key, result));
                context.request_repaint();
            }
        });
        Self {
            results: HashMap::new(),
            order: VecDeque::new(),
            pending: HashSet::new(),
            jobs,
            receiver,
        }
    }
}

impl HighlightCache {
    fn spans(
        &mut self,
        context: &Context,
        payload: &str,
        active_plugin_ids: &[String],
        highlighter: Option<&PayloadHighlighter>,
    ) -> HighlightResult {
        self.collect();
        let key = HighlightKey::new(payload, active_plugin_ids);
        if let Some(result) = self.results.get(&key) {
            return result.clone();
        }
        let highlighter = highlighter?;
        if self.pending.len() < HIGHLIGHT_WORKER_CAPACITY
            && self
                .jobs
                .try_send((
                    key,
                    payload.to_owned(),
                    active_plugin_ids.to_vec(),
                    context.clone(),
                    highlighter.clone(),
                ))
                .is_ok()
        {
            self.pending.insert(key);
        }
        None
    }

    fn collect(&mut self) {
        while let Ok((key, result)) = self.receiver.try_recv() {
            self.pending.remove(&key);
            if self.results.insert(key, result).is_none() {
                self.order.push_back(key);
            }
        }
        while self.order.len() > HIGHLIGHT_CACHE_CAPACITY {
            if let Some(key) = self.order.pop_front() {
                self.results.remove(&key);
            }
        }
    }
}

pub(crate) fn cached_spans(
    ui: &Ui,
    payload: &str,
    active_plugin_ids: &[String],
    highlighter: Option<&PayloadHighlighter>,
) -> HighlightResult {
    let cache = ui.ctx().data_mut(|data| {
        data.get_temp::<Arc<Mutex<HighlightCache>>>(Id::new(HIGHLIGHT_CACHE_ID))
            .unwrap_or_else(|| {
                let cache = Arc::new(Mutex::new(HighlightCache::default()));
                data.insert_temp(Id::new(HIGHLIGHT_CACHE_ID), cache.clone());
                cache
            })
    });
    cache
        .lock()
        .ok()
        .and_then(|mut cache| cache.spans(ui.ctx(), payload, active_plugin_ids, highlighter))
}

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

pub(crate) fn layouter(spans: HighlightResult) -> impl FnMut(&Ui, &str, f32) -> Arc<egui::Galley> {
    move |ui, text, wrap_width| {
        let mut job = highlight_payload(ui, text, spans.as_deref());
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

fn highlight_payload(ui: &Ui, text: &str, spans: Option<&[PayloadSyntaxSpan]>) -> LayoutJob {
    let font = TextStyle::Monospace.resolve(ui.style());
    let palette = palette(ui);
    if let Some(spans) = spans {
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
    spans: &[PayloadSyntaxSpan],
    font: FontId,
    palette: Palette,
) -> LayoutJob {
    let mut job = LayoutJob::default();
    let mut index = 0;
    for span in spans {
        if span.start < index
            || span.start >= span.end
            || span.end > text.len()
            || !text.is_char_boundary(span.start)
            || !text.is_char_boundary(span.end)
        {
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

#[cfg(test)]
#[path = "payload_highlight_tests.rs"]
mod tests;
