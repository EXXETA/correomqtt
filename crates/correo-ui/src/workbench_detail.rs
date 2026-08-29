use correo_core::{AppCommand, AppCommandSender, AppSnapshot, MessageRow, PublishHistoryRow};
use correo_style::layout;
use egui::{Button, Label, Layout, RichText, TextEdit, Ui, WidgetInfo, WidgetType};
use egui_phosphor::regular;

use crate::{
    i18n::I18n,
    payload_highlight,
    theme::ThemeTokens,
    widgets::{padded_text_edit, square_icon_button_size, with_icon_button_padding},
    workbench_connection_messages_text::formatted_size,
    PayloadHighlighter,
};

const DETAIL_TOOLBAR_HEIGHT: f32 = 48.0;

pub(crate) fn message_window_content(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    message: &MessageRow,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
    payload_highlighter: Option<&PayloadHighlighter>,
) {
    detail_view(
        ui,
        MessageDetail {
            topic: &message.topic,
            timestamp: &message.timestamp,
            qos: message.qos.label(),
            retained: message.retained,
            byte_size: message.byte_size,
            payload: &message.payload,
            fallback_payload: &message.payload_preview,
            diagnostics: &message.diagnostics,
            export: AppCommand::ExportIncomingMessage(message.id),
            active_plugin_ids: snapshot.plugins.active_plugin_ids(),
        },
        tokens,
        commands,
        i18n,
        payload_highlighter,
    );
}

pub(crate) fn outgoing_window_content(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    row: &PublishHistoryRow,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
    payload_highlighter: Option<&PayloadHighlighter>,
) {
    detail_view(
        ui,
        MessageDetail {
            topic: &row.topic,
            timestamp: &row.timestamp,
            qos: row.qos.label(),
            retained: row.retained,
            byte_size: row.byte_size,
            payload: &row.payload,
            fallback_payload: &row.payload_preview,
            diagnostics: &[],
            export: AppCommand::ExportPublishHistoryMessage(row.id),
            active_plugin_ids: snapshot.plugins.active_plugin_ids(),
        },
        tokens,
        commands,
        i18n,
        payload_highlighter,
    );
}

fn detail_view(
    ui: &mut Ui,
    detail: MessageDetail<'_>,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
    payload_highlighter: Option<&PayloadHighlighter>,
) {
    detail_toolbar(ui, &detail, tokens, commands, i18n);
    ui.add_space(6.0);
    validation_status(ui, &detail, tokens);
    payload_area(ui, &detail, payload_highlighter);
}

fn validation_status(ui: &mut Ui, detail: &MessageDetail<'_>, tokens: ThemeTokens) {
    for diagnostic in detail
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.plugin_id.is_some())
    {
        let color = match diagnostic.severity {
            correo_core::PluginDiagnosticSeverity::Info => tokens.success,
            correo_core::PluginDiagnosticSeverity::Warning => tokens.warning,
            correo_core::PluginDiagnosticSeverity::Error => tokens.danger,
        };
        ui.label(RichText::new(&diagnostic.message).small().color(color));
    }
}

fn detail_toolbar(
    ui: &mut Ui,
    detail: &MessageDetail<'_>,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), DETAIL_TOOLBAR_HEIGHT),
        Layout::left_to_right(egui::Align::Center),
        |ui| {
            let button_width = (square_icon_button_size()[0] + layout::TOOLBAR_GAP) * 4.0;
            ui.allocate_ui_with_layout(
                egui::vec2(
                    (ui.available_width() - button_width).max(0.0),
                    DETAIL_TOOLBAR_HEIGHT,
                ),
                Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.add(Label::new(RichText::new(detail.topic).strong()).truncate());
                    ui.add_space(2.0);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;
                        ui.label(
                            RichText::new(crate::time_format::local_date_time(detail.timestamp))
                                .color(tokens.text_secondary),
                        );
                        ui.label(RichText::new(detail.qos).color(tokens.text_secondary));
                        ui.label(
                            RichText::new(formatted_size(detail.byte_size))
                                .color(tokens.text_secondary),
                        );
                        if detail.retained {
                            ui.label(
                                RichText::new(i18n.text("message-retained"))
                                    .color(tokens.accent)
                                    .strong(),
                            );
                        }
                    });
                },
            );
            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                if detail_icon_button(
                    ui,
                    regular::DOWNLOAD_SIMPLE,
                    &i18n.text("message-action-save"),
                )
                .clicked()
                {
                    if let Some(path) = save_message_path(detail.topic) {
                        send(commands, export_command_to_path(&detail.export, path));
                    }
                }
                if detail_icon_button(
                    ui,
                    regular::CLIPBOARD_TEXT,
                    &i18n.text("message-action-copy-payload"),
                )
                .clicked()
                {
                    ui.ctx()
                        .copy_text(payload_text(detail.payload, detail.fallback_payload));
                }
                if detail_icon_button(ui, regular::CLOCK, &i18n.text("message-action-copy-time"))
                    .clicked()
                {
                    ui.ctx().copy_text(detail.timestamp.to_owned());
                }
                if detail_icon_button(ui, regular::COPY, &i18n.text("message-action-copy-topic"))
                    .clicked()
                {
                    ui.ctx().copy_text(detail.topic.to_owned());
                }
            });
        },
    );
}

fn detail_icon_button(ui: &mut Ui, icon: &str, hover_text: &str) -> egui::Response {
    let response = with_icon_button_padding(ui, |ui| {
        ui.add_sized(
            square_icon_button_size(),
            Button::new(RichText::new(icon).size(16.0)),
        )
    });
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, hover_text));
    response.on_hover_text(hover_text)
}

fn payload_area(
    ui: &mut Ui,
    detail: &MessageDetail<'_>,
    payload_highlighter: Option<&PayloadHighlighter>,
) {
    let mut payload = payload_text(detail.payload, detail.fallback_payload);
    let height = ui.available_height().max(0.0);
    let spans = payload_highlight::cached_spans(
        ui,
        &payload,
        &detail.active_plugin_ids,
        payload_highlighter,
    );
    let mut layouter = payload_highlight::layouter(spans);
    ui.add_sized(
        [ui.available_width(), height],
        padded_text_edit(
            TextEdit::multiline(&mut payload)
                .font(egui::TextStyle::Monospace)
                .desired_width(f32::INFINITY)
                .layouter(&mut layouter),
        ),
    );
}

fn payload_text(payload: &[u8], fallback: &str) -> String {
    if payload.is_empty() {
        fallback.to_owned()
    } else {
        String::from_utf8_lossy(payload).into_owned()
    }
}

struct MessageDetail<'a> {
    topic: &'a str,
    timestamp: &'a str,
    qos: &'a str,
    retained: bool,
    byte_size: usize,
    payload: &'a [u8],
    fallback_payload: &'a str,
    diagnostics: &'a [correo_core::MessageDiagnosticRow],
    export: AppCommand,
    active_plugin_ids: Vec<String>,
}

fn send(commands: &AppCommandSender, command: AppCommand) {
    let _ = commands.send(command);
}

fn export_command_to_path(command: &AppCommand, path: std::path::PathBuf) -> AppCommand {
    match command {
        AppCommand::ExportPublishHistoryMessage(message_id) => {
            AppCommand::ExportPublishHistoryMessageToPath {
                message_id: *message_id,
                path,
            }
        }
        AppCommand::ExportIncomingMessage(message_id) => AppCommand::ExportIncomingMessageToPath {
            message_id: *message_id,
            path,
        },
        _ => command.clone(),
    }
}

fn save_message_path(topic: &str) -> Option<std::path::PathBuf> {
    rfd::FileDialog::new()
        .add_filter("CorreoMQTT message", &["cqm"])
        .set_file_name(suggested_message_file_name(topic))
        .save_file()
}

fn suggested_message_file_name(topic: &str) -> String {
    let name = topic
        .chars()
        .map(|character| match character {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => character,
            '/' | '.' | ':' => '-',
            _ => '_',
        })
        .collect::<String>();
    let name = name.trim_matches(['-', '_']).trim();
    if name.is_empty() {
        "message.cqm".to_owned()
    } else {
        format!("{name}.cqm")
    }
}
