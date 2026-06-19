use correo_core::{AppCommand, AppCommandSender, ScriptLogLevel, ScriptSurfaceSnapshot};
use egui::{text::LayoutJob, Button, FontId, ScrollArea, TextFormat, TextStyle, Ui};

use crate::i18n::I18n;
use crate::theme::ThemeTokens;

pub(super) fn log_view(
    ui: &mut Ui,
    scripts: &ScriptSurfaceSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    let selected_execution_id = scripts.selected_execution_id();
    ui.horizontal(|ui| {
        ui.heading(i18n.text("script-execution-log"));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let can_stop = scripts.running_execution_id().is_some();
            if ui
                .add_enabled(can_stop, Button::new(i18n.text("script-stop")))
                .on_hover_text(i18n.text("script-stop-tooltip"))
                .clicked()
            {
                let _ = commands.send(AppCommand::CancelScript);
            }
        });
    });
    ui.add_space(4.0);
    ScrollArea::vertical()
        .id_salt("script-log")
        .stick_to_bottom(true)
        .show(ui, |ui| {
            for line in scripts.log_lines.iter().filter(|line| {
                selected_execution_id.is_none_or(|execution_id| line.execution_id == execution_id)
            }) {
                ui.label(log_line_job(
                    ui,
                    &line.timestamp,
                    line.level,
                    &line.message,
                    tokens,
                ));
            }
        });
}

fn log_line_job(
    ui: &Ui,
    timestamp: &str,
    level: ScriptLogLevel,
    message: &str,
    tokens: ThemeTokens,
) -> LayoutJob {
    let font = TextStyle::Monospace.resolve(ui.style());
    let mut job = LayoutJob::default();
    let timestamp = crate::time_format::local_time(timestamp);
    append(&mut job, &timestamp, font.clone(), tokens.text_secondary);
    append(&mut job, " ", font.clone(), tokens.text_secondary);
    append(
        &mut job,
        level.label(),
        font.clone(),
        log_color(level, tokens),
    );
    append(&mut job, " ", font.clone(), tokens.text_secondary);
    append_message(&mut job, message, font, tokens);
    job
}

fn append_message(job: &mut LayoutJob, message: &str, font: FontId, tokens: ThemeTokens) {
    let mut rest = message;
    while !rest.is_empty() {
        if let Some(end) = script_exec_prefix_end(rest) {
            append(job, &rest[..end], font.clone(), tokens.script);
            rest = &rest[end..];
        } else if let Some((before, file, after)) = next_filename(rest) {
            append(job, before, font.clone(), tokens.text_primary);
            append(job, file, font.clone(), tokens.accent);
            rest = after;
        } else {
            append(job, rest, font.clone(), tokens.text_primary);
            break;
        }
    }
}

fn script_exec_prefix_end(text: &str) -> Option<usize> {
    let remainder = text.strip_prefix("[script-exec-")?;
    let end = remainder.find(']')? + "[script-exec-".len() + 1;
    Some(end)
}

fn next_filename(text: &str) -> Option<(&str, &str, &str)> {
    let start = text.find(".js")?;
    let end = start + ".js".len();
    let name_start = text[..start]
        .rfind(|character: char| character.is_whitespace() || character == '[' || character == '(')
        .map_or(0, |index| index + 1);
    let before = &text[..name_start];
    let file = &text[name_start..end];
    let after = &text[end..];
    (!file.is_empty()).then_some((before, file, after))
}

fn append(job: &mut LayoutJob, text: &str, font: FontId, color: egui::Color32) {
    job.append(text, 0.0, TextFormat::simple(font, color));
}

fn log_color(level: ScriptLogLevel, tokens: ThemeTokens) -> egui::Color32 {
    match level {
        ScriptLogLevel::Debug => tokens.text_secondary,
        ScriptLogLevel::Info => tokens.success,
        ScriptLogLevel::Warning => tokens.warning,
        ScriptLogLevel::Error => tokens.danger,
    }
}
