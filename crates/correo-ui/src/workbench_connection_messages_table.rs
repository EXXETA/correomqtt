use correo_core::{AppCommand, AppCommandSender, AppSnapshot, MessageRow, PublishHistoryRow};
use correo_style::layout;
use egui::{Button, Id, Rect, RichText, ScrollArea, Sense, Ui, WidgetInfo, WidgetType};
use egui_phosphor::regular;

use crate::{
    i18n::I18n,
    theme::ThemeTokens,
    widgets::{
        clearable_search_edit, dotted_focus_outline, fill_remaining_tile_rows, menu_item,
        paint_focus_outline, set_menu_item_width, square_icon_button_size,
        tile_scroll_bar_rect_with_height, tile_table_interactive_fill, tile_table_selected_fill,
        with_icon_button_padding,
    },
    workbench_connection_messages_filters::{message_visible_for_subscriptions, row_matches},
    workbench_connection_messages_text::{
        formatted_size, middle_ellipsis, right_aligned_text, text_width, truncated_text,
    },
    workbench_helpers::send,
    workbench_messages,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MessageOrigin {
    Outgoing,
    Incoming,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MessageKey {
    Outgoing(u32),
    Incoming(u32),
}

struct ConnectionMessageRow<'a> {
    key: MessageKey,
    topic: &'a str,
    timestamp: &'a str,
    qos: &'a str,
    retained: bool,
    payload_preview: &'a str,
    plugin_diagnostic: Option<&'a correo_core::MessageDiagnosticRow>,
    validation_status: Option<ValidationStatus>,
    byte_size: usize,
    selected: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ValidationStatus {
    Validated,
    Invalid,
}

#[derive(Clone, Copy)]
struct MessageRowMetrics {
    height: f32,
    topic_y: f32,
    preview_y: f32,
}

fn message_row_metrics(ui: &Ui) -> MessageRowMetrics {
    const VERTICAL_PADDING: f32 = 6.0;
    const LINE_GAP: f32 = 2.0;
    let topic_font = egui::TextStyle::Button.resolve(ui.style());
    let meta_font = egui::TextStyle::Small.resolve(ui.style());
    let (topic_height, meta_height) = ui.fonts(|fonts| {
        (
            fonts.row_height(&topic_font),
            fonts.row_height(&meta_font),
        )
    });
    MessageRowMetrics {
        height: (VERTICAL_PADDING * 2.0 + topic_height + LINE_GAP + meta_height).ceil(),
        topic_y: VERTICAL_PADDING,
        preview_y: VERTICAL_PADDING + topic_height + LINE_GAP,
    }
}

pub(crate) fn show(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    origin: MessageOrigin,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    toolbar(ui, snapshot, origin, tokens, commands, i18n);
    ui.add_space(4.0);

    let rows = rows(snapshot, origin);
    message_table(
        ui,
        snapshot,
        origin,
        &rows,
        auto_scroll_enabled(ui, origin),
        tokens,
        commands,
        i18n,
    );
}

fn toolbar(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    origin: MessageOrigin,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    let selected = selected_key(snapshot, origin);
    let toolbar_width = ui.available_width();
    ui.set_width(toolbar_width);
    ui.horizontal(|ui| {
        ui.set_width(toolbar_width);
        if icon_button(
            ui,
            regular::UPLOAD_SIMPLE,
            &i18n.text("message-action-copy-to-publish"),
            selected.is_some(),
            false,
            tokens,
        )
        .clicked()
        {
            if let Some(key) = selected {
                send(commands, copy_command(key));
            }
        }

        if icon_button(
            ui,
            regular::SHARE,
            &i18n.text("message-action-open-window"),
            selected.is_some(),
            false,
            tokens,
        )
        .clicked()
        {
            if let Some(key) = selected {
                open_message(ui, snapshot, key);
            }
        }

        let mut filter = filter_text(snapshot, origin).to_owned();
        let button_width = layout::square_icon_button_side();
        let search_width = (toolbar_width - button_width * 4.0 - ui.spacing().item_spacing.x * 4.0)
            .max(button_width);
        let search_hint = search_hint(origin, i18n);
        if clearable_search_edit(ui, None, &mut filter, &search_hint, search_width).changed() {
            send(commands, search_command(origin, filter));
        }

        if icon_button(
            ui,
            regular::TRASH,
            &i18n.text("message-action-clear"),
            source_has_messages(snapshot, origin),
            false,
            tokens,
        )
        .clicked()
        {
            send(commands, clear_command(origin));
        }

        let auto_scroll = auto_scroll_enabled(ui, origin);
        if icon_button(
            ui,
            regular::MOUSE_SCROLL,
            &i18n.text("message-action-auto-scroll"),
            true,
            auto_scroll,
            tokens,
        )
        .clicked()
        {
            set_auto_scroll_enabled(ui, origin, !auto_scroll);
        }
    });
}

fn icon_button(
    ui: &mut Ui,
    icon: &str,
    hover_text: &str,
    enabled: bool,
    active: bool,
    tokens: ThemeTokens,
) -> egui::Response {
    let mut button = Button::new(RichText::new(icon).size(16.0));
    if active {
        button = button
            .fill(tile_table_selected_fill(tokens))
            .stroke(egui::Stroke::NONE);
    }
    let response = ui
        .add_enabled_ui(enabled, |ui| {
            with_icon_button_padding(ui, |ui| ui.add_sized(square_icon_button_size(), button))
        })
        .inner;
    paint_focus_outline(ui, &response);
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, enabled, hover_text));
    response.on_hover_text(hover_text)
}

fn filter_text(snapshot: &AppSnapshot, origin: MessageOrigin) -> &str {
    match origin {
        MessageOrigin::Outgoing => &snapshot.workbench.publish.history_filter,
        MessageOrigin::Incoming => &snapshot.workbench.subscribe.message_filter,
    }
}

fn search_hint(origin: MessageOrigin, i18n: &I18n) -> String {
    i18n.text(match origin {
        MessageOrigin::Outgoing => "message-search-outgoing",
        MessageOrigin::Incoming => "message-search-incoming",
    })
}

fn search_command(origin: MessageOrigin, filter: String) -> AppCommand {
    match origin {
        MessageOrigin::Outgoing => AppCommand::SearchPublishHistory(filter),
        MessageOrigin::Incoming => AppCommand::SearchMessages(filter),
    }
}

fn rows(snapshot: &AppSnapshot, origin: MessageOrigin) -> Vec<ConnectionMessageRow<'_>> {
    let filter = filter_text(snapshot, origin).to_ascii_lowercase();
    match origin {
        MessageOrigin::Outgoing => snapshot
            .workbench
            .publish
            .history
            .iter()
            .filter(|row| row_matches(row.topic.as_str(), row.payload_preview.as_str(), &filter))
            .map(|row| outgoing_row(snapshot, row))
            .collect(),
        MessageOrigin::Incoming => snapshot
            .workbench
            .messages
            .iter()
            .filter(|message| message_visible_for_subscriptions(message, snapshot))
            .filter(|message| row_matches(&message.topic, &message.payload_preview, &filter))
            .map(|message| incoming_row(snapshot, message))
            .collect(),
    }
}

fn outgoing_row<'a>(
    snapshot: &AppSnapshot,
    row: &'a PublishHistoryRow,
) -> ConnectionMessageRow<'a> {
    ConnectionMessageRow {
        key: MessageKey::Outgoing(row.id),
        topic: &row.topic,
        timestamp: &row.timestamp,
        qos: row.qos.label(),
        retained: row.retained,
        payload_preview: &row.payload_preview,
        plugin_diagnostic: row.diagnostics.iter().find(|diagnostic| {
            diagnostic.plugin_id.is_some() && diagnostic_is_attention(diagnostic)
        }),
        validation_status: validation_status_for_row(&row.diagnostics),
        byte_size: row.byte_size,
        selected: snapshot.workbench.publish.selected_history_id == Some(row.id),
    }
}

fn incoming_row<'a>(snapshot: &AppSnapshot, message: &'a MessageRow) -> ConnectionMessageRow<'a> {
    let plugin_diagnostic = message
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.plugin_id.is_some() && diagnostic_is_attention(diagnostic));
    ConnectionMessageRow {
        key: MessageKey::Incoming(message.id),
        topic: &message.topic,
        timestamp: &message.timestamp,
        qos: message.qos.label(),
        retained: message.retained,
        payload_preview: &message.payload_preview,
        plugin_diagnostic,
        validation_status: validation_status_for_row(&message.diagnostics),
        byte_size: message.byte_size,
        selected: snapshot.workbench.selected_message_id == Some(message.id),
    }
}

fn validation_status_for_row(
    diagnostics: &[correo_core::MessageDiagnosticRow],
) -> Option<ValidationStatus> {
    let mut saw_validator_diagnostic = false;
    for diagnostic in diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.hook == Some(correo_core::PluginHookKind::Validator))
    {
        saw_validator_diagnostic = true;
        if diagnostic.severity == correo_core::PluginDiagnosticSeverity::Error {
            return Some(ValidationStatus::Invalid);
        }
    }
    saw_validator_diagnostic.then_some(ValidationStatus::Validated)
}

fn diagnostic_is_attention(diagnostic: &correo_core::MessageDiagnosticRow) -> bool {
    diagnostic.severity != correo_core::PluginDiagnosticSeverity::Info
}

fn message_table(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    origin: MessageOrigin,
    rows: &[ConnectionMessageRow<'_>],
    auto_scroll: bool,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    ui.spacing_mut().item_spacing.y = 0.0;
    let table_height = ui
        .available_rect_before_wrap()
        .height()
        .max(layout::TABLE_MIN_HEIGHT);
    let table_id = message_table_focus_id(origin);
    let table_rect = egui::Rect::from_min_size(
        ui.available_rect_before_wrap().min,
        egui::vec2(ui.available_width(), table_height),
    );
    let table_response = ui.interact(table_rect, table_id, Sense::focusable_noninteractive());
    if table_response.has_focus() {
        ui.memory_mut(|memory| {
            memory.set_focus_lock_filter(
                table_id,
                egui::EventFilter {
                    vertical_arrows: true,
                    ..Default::default()
                },
            );
        });
        handle_message_table_keyboard(ui, snapshot, origin, rows, commands);
    }
    if table_response.gained_focus() {
        store_message_focus_index(ui, origin, selected_message_index(rows).unwrap_or(0));
    }
    let focused_index = message_focus_index(ui, origin, rows);
    let row_metrics = message_row_metrics(ui);
    ScrollArea::vertical()
        .id_salt(match origin {
            MessageOrigin::Outgoing => "outgoing-messages-table",
            MessageOrigin::Incoming => "incoming-messages-table",
        })
        .max_height(table_height)
        .scroll_bar_rect(tile_scroll_bar_rect_with_height(ui, table_height))
        .stick_to_bottom(auto_scroll)
        .auto_shrink([false, false])
        .show_rows(
            ui,
            row_metrics.height,
            rows.len(),
            |ui, row_range| {
                ui.set_width(ui.available_width());
                for index in row_range {
                    if let Some(row) = rows.get(index) {
                        message_row(
                            ui,
                            snapshot,
                            origin,
                            index,
                            row,
                            tokens,
                            commands,
                            table_response.has_focus() && index == focused_index,
                            i18n,
                            row_metrics,
                        );
                    }
                }
                fill_remaining_tile_rows(
                    ui,
                    rows.len(),
                    row_metrics.height,
                    table_height,
                    tokens,
                );
            },
        );
}
