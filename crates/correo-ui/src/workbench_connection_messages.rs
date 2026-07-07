use correo_core::{AppCommand, AppCommandSender, AppSnapshot, MessageRow, PublishHistoryRow};
use correo_style::layout;
use egui::{Button, Id, Rect, RichText, ScrollArea, Sense, Ui};
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
    workbench_connection_messages_filters::{
        message_visible_for_subscriptions, row_matches, topic_matches_filter,
    },
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

pub(crate) fn show(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    origin: MessageOrigin,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    toolbar(ui, snapshot, origin, tokens, commands);
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
) {
    let selected = selected_key(snapshot, origin);
    let toolbar_width = ui.available_width();
    ui.set_width(toolbar_width);
    ui.horizontal(|ui| {
        ui.set_width(toolbar_width);
        if icon_button(
            ui,
            regular::UPLOAD_SIMPLE,
            "Copy selected message to publish form",
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
            "Show selected message in extra window",
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
        if clearable_search_edit(ui, None, &mut filter, search_hint(origin), search_width).changed()
        {
            send(commands, search_command(origin, filter));
        }

        if icon_button(
            ui,
            regular::TRASH,
            "Clear messages",
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
            "Toggle automatic scrolling",
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
    response.on_hover_text(hover_text)
}

fn filter_text(snapshot: &AppSnapshot, origin: MessageOrigin) -> &str {
    match origin {
        MessageOrigin::Outgoing => &snapshot.workbench.publish.history_filter,
        MessageOrigin::Incoming => &snapshot.workbench.subscribe.message_filter,
    }
}

fn search_hint(origin: MessageOrigin) -> &'static str {
    match origin {
        MessageOrigin::Outgoing => "Search outgoing",
        MessageOrigin::Incoming => "Search incoming",
    }
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
        validation_status: validation_status_for_row(
            snapshot,
            &row.topic,
            &row.payload,
            MessageOrigin::Outgoing,
            &row.diagnostics,
        ),
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
        validation_status: validation_status_for_row(
            snapshot,
            &message.topic,
            &message.payload,
            MessageOrigin::Incoming,
            &message.diagnostics,
        ),
        byte_size: message.byte_size,
        selected: snapshot.workbench.selected_message_id == Some(message.id),
    }
}

fn validation_status_for_row(
    snapshot: &AppSnapshot,
    topic: &str,
    payload: &[u8],
    origin: MessageOrigin,
    diagnostics: &[correo_core::MessageDiagnosticRow],
) -> Option<ValidationStatus> {
    for diagnostic in diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.hook == Some(correo_core::PluginHookKind::Validator))
    {
        if diagnostic.severity == correo_core::PluginDiagnosticSeverity::Error {
            return Some(ValidationStatus::Invalid);
        }
    }
    let mut matched = false;
    for workflow in snapshot
        .connection_settings
        .plugin_workflows
        .iter()
        .filter(|workflow| {
            workflow.enabled
                && workflow.kind == correo_core::ConnectionPluginWorkflowKind::Validator
                && workflow_direction_matches(workflow.direction, origin)
                && topic_matches_filter(topic, &workflow.topic_filter)
        })
    {
        matched = true;
        if !workflow_validates(workflow, payload) {
            return Some(ValidationStatus::Invalid);
        }
    }
    matched.then_some(ValidationStatus::Validated)
}

fn workflow_validates(workflow: &correo_core::ConnectionPluginWorkflow, payload: &[u8]) -> bool {
    match workflow.plugin_id.as_str() {
        "org.correomqtt.plugins.contains-string-validator" => {
            contains_string_validator_validates(&workflow.config, payload)
        }
        "org.correomqtt.plugins.xml-xsd-validator" => {
            let payload = String::from_utf8_lossy(payload);
            let xsd_path = workflow
                .config
                .get("xsd_path")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default();
            payload.trim_start().starts_with('<') && !xsd_path.trim().is_empty()
        }
        _ => true,
    }
}

fn contains_string_validator_validates(config: &serde_json::Value, payload: &[u8]) -> bool {
    let payload = String::from_utf8_lossy(payload);
    let rules = config
        .get("rules")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    rules.is_empty()
        || rules.iter().any(|rule| {
            let Some(needle) = rule.get("text").and_then(serde_json::Value::as_str) else {
                return false;
            };
            if rule
                .get("regex")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false)
            {
                regex::Regex::new(needle)
                    .map(|regex| regex.is_match(&payload))
                    .unwrap_or(false)
            } else {
                payload.contains(needle)
            }
        })
}

fn workflow_direction_matches(
    direction: correo_core::ConnectionPluginDirection,
    origin: MessageOrigin,
) -> bool {
    direction == correo_core::ConnectionPluginDirection::Both
        || matches!(
            (direction, origin),
            (
                correo_core::ConnectionPluginDirection::Outgoing,
                MessageOrigin::Outgoing
            ) | (
                correo_core::ConnectionPluginDirection::Incoming,
                MessageOrigin::Incoming
            )
        )
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
            layout::MESSAGE_TABLE_ROW_HEIGHT,
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
                            table_response.has_focus(),
                            focused_index,
                            i18n,
                        );
                    }
                }
                fill_remaining_tile_rows(
                    ui,
                    rows.len(),
                    layout::MESSAGE_TABLE_ROW_HEIGHT,
                    table_height,
                    tokens,
                );
            },
        );
}

fn message_row(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    origin: MessageOrigin,
    index: usize,
    row: &ConnectionMessageRow<'_>,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    table_focused: bool,
    focused_index: usize,
    i18n: &I18n,
) {
    let row_width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(row_width, layout::MESSAGE_TABLE_ROW_HEIGHT),
        Sense::CLICK,
    );
    let row_focused = table_focused && index == focused_index;
    let fill = tile_table_interactive_fill(
        index,
        tokens,
        response.hovered() || row_focused,
        row.selected,
    );
    ui.painter()
        .rect_filled(rect, egui::CornerRadius::ZERO, fill);
    if row_focused {
        dotted_focus_outline(ui, rect);
    }

    response.context_menu(|ui| message_context_menu(ui, snapshot, origin, row, commands));
    if response.clicked() {
        ui.memory_mut(|memory| memory.request_focus(message_table_focus_id(origin)));
        store_message_focus_index(ui, origin, index);
        send(commands, select_command(row.key));
    }
    if response.double_clicked() {
        send(commands, select_command(row.key));
        open_message(ui, snapshot, row.key);
    }

    let right = rect.right() - layout::MESSAGE_ROW_PADDING_RIGHT;
    let meta_rect = right_rect(rect, right, layout::MESSAGE_ROW_META_WIDTH);
    let topic_y = rect.top() + 7.0;
    let preview_y = rect.top() + 28.0;
    let topic_font = egui::TextStyle::Button.resolve(ui.style());
    let meta_font = egui::TextStyle::Small.resolve(ui.style());
    let timestamp = crate::time_format::local_date_time(row.timestamp);
    let size = formatted_size(row.byte_size);
    let qos_and_size = if row.retained {
        format!("Retained · {} · {size}", row.qos)
    } else {
        format!("{} · {size}", row.qos)
    };
    let validation_label = row.validation_status.map(|status| match status {
        ValidationStatus::Validated => i18n.text("message-validation-validated"),
        ValidationStatus::Invalid => i18n.text("message-validation-invalid"),
    });
    let bottom_meta = validation_label
        .as_deref()
        .map(|label| format!("{label} · {qos_and_size}"))
        .unwrap_or_else(|| qos_and_size.clone());
    let preview_meta_width = text_width(ui, &bottom_meta, meta_font.clone());
    let topic_left = rect.left() + layout::SUBSCRIPTION_ROW_PADDING_X;
    let topic_right =
        right - text_width(ui, &timestamp, meta_font.clone()) - layout::MESSAGE_ROW_TOPIC_META_GAP;
    let preview_right = right - preview_meta_width - layout::MESSAGE_ROW_TOPIC_META_GAP;
    let topic_width = (topic_right - topic_left).max(0.0);
    let preview_width = (preview_right - topic_left).max(0.0);
    let topic = middle_ellipsis(ui, row.topic, topic_font.clone(), topic_width);
    ui.painter().text(
        egui::pos2(topic_left, topic_y),
        egui::Align2::LEFT_TOP,
        topic,
        topic_font,
        ui.visuals().text_color(),
    );
    if let Some(diagnostic) = row.plugin_diagnostic {
        truncated_text(
            ui,
            egui::pos2(topic_left, preview_y),
            preview_width,
            &diagnostic.message,
            meta_font.clone(),
            match diagnostic.severity {
                correo_core::PluginDiagnosticSeverity::Info => tokens.success,
                correo_core::PluginDiagnosticSeverity::Warning => tokens.warning,
                correo_core::PluginDiagnosticSeverity::Error => tokens.danger,
            },
        );
    } else {
        truncated_text(
            ui,
            egui::pos2(topic_left, preview_y),
            preview_width,
            row.payload_preview,
            meta_font.clone(),
            tokens.text_secondary,
        );
    }
    right_aligned_text(
        ui,
        meta_rect.right_top() + egui::vec2(0.0, 7.0),
        &timestamp,
        ui.visuals().text_color(),
    );
    right_aligned_text(
        ui,
        meta_rect.right_top() + egui::vec2(0.0, 28.0),
        &qos_and_size,
        tokens.text_secondary,
    );
    if let Some((status, label)) = row.validation_status.zip(validation_label.as_deref()) {
        let dot_gap = text_width(ui, " · ", meta_font.clone());
        let x = meta_rect.right() - text_width(ui, &qos_and_size, meta_font.clone()) - dot_gap;
        right_aligned_text(
            ui,
            egui::pos2(x, meta_rect.top() + 28.0),
            "·",
            tokens.text_secondary,
        );
        let x = x - dot_gap;
        right_aligned_text(
            ui,
            egui::pos2(x, meta_rect.top() + 28.0),
            label,
            match status {
                ValidationStatus::Validated => tokens.success,
                ValidationStatus::Invalid => tokens.danger,
            },
        );
    }
}

fn handle_message_table_keyboard(
    ui: &Ui,
    snapshot: &AppSnapshot,
    origin: MessageOrigin,
    rows: &[ConnectionMessageRow<'_>],
    commands: &AppCommandSender,
) {
    if rows.is_empty() {
        return;
    }
    let current = message_focus_index(ui, origin, rows);
    let next = ui.input(|input| {
        if input.key_pressed(egui::Key::ArrowDown) {
            Some((current + 1).min(rows.len() - 1))
        } else if input.key_pressed(egui::Key::ArrowUp) {
            Some(current.saturating_sub(1))
        } else {
            None
        }
    });
    if let Some(next) = next {
        store_message_focus_index(ui, origin, next);
        return;
    }
    if ui.input(|input| input.key_pressed(egui::Key::Enter)) {
        if let Some(row) = rows.get(current) {
            send(commands, select_command(row.key));
            open_message(ui, snapshot, row.key);
        }
    } else if ui.input(|input| input.key_pressed(egui::Key::Space)) {
        if let Some(row) = rows.get(current) {
            send(commands, select_command(row.key));
        }
    }
}

fn message_focus_index(ui: &Ui, origin: MessageOrigin, rows: &[ConnectionMessageRow<'_>]) -> usize {
    let selected = selected_message_index(rows).unwrap_or(0);
    ui.ctx().data_mut(|data| {
        data.get_temp::<usize>(message_focus_index_id(origin))
            .unwrap_or(selected)
            .min(rows.len().saturating_sub(1))
    })
}

fn store_message_focus_index(ui: &Ui, origin: MessageOrigin, index: usize) {
    ui.ctx()
        .data_mut(|data| data.insert_temp(message_focus_index_id(origin), index));
}

fn selected_message_index(rows: &[ConnectionMessageRow<'_>]) -> Option<usize> {
    rows.iter().position(|row| row.selected)
}

fn message_table_focus_id(origin: MessageOrigin) -> Id {
    match origin {
        MessageOrigin::Outgoing => Id::new("outgoing-messages-table-focus"),
        MessageOrigin::Incoming => Id::new("incoming-messages-table-focus"),
    }
}

fn message_focus_index_id(origin: MessageOrigin) -> Id {
    match origin {
        MessageOrigin::Outgoing => Id::new("outgoing-messages-table-focused-row"),
        MessageOrigin::Incoming => Id::new("incoming-messages-table-focused-row"),
    }
}

fn right_rect(row: Rect, right: f32, width: f32) -> Rect {
    Rect::from_min_max(
        egui::pos2((right - width).max(row.left()), row.top()),
        egui::pos2(right, row.bottom()),
    )
}

fn select_command(key: MessageKey) -> AppCommand {
    match key {
        MessageKey::Outgoing(id) => AppCommand::SelectPublishHistoryMessage(id),
        MessageKey::Incoming(id) => AppCommand::SelectMessage(id),
    }
}

fn copy_command(key: MessageKey) -> AppCommand {
    match key {
        MessageKey::Outgoing(id) => AppCommand::CopyPublishHistoryMessageToPublishForm(id),
        MessageKey::Incoming(id) => AppCommand::CopyIncomingMessageToPublishForm(id),
    }
}

fn clear_command(origin: MessageOrigin) -> AppCommand {
    match origin {
        MessageOrigin::Outgoing => AppCommand::ClearPublishHistory,
        MessageOrigin::Incoming => AppCommand::ClearIncomingMessages,
    }
}

fn export_command_to_path(key: MessageKey, path: std::path::PathBuf) -> AppCommand {
    match key {
        MessageKey::Outgoing(message_id) => {
            AppCommand::ExportPublishHistoryMessageToPath { message_id, path }
        }
        MessageKey::Incoming(message_id) => {
            AppCommand::ExportIncomingMessageToPath { message_id, path }
        }
    }
}

fn remove_command(key: MessageKey) -> AppCommand {
    match key {
        MessageKey::Outgoing(id) => AppCommand::RemovePublishHistoryMessage(id),
        MessageKey::Incoming(id) => AppCommand::RemoveIncomingMessage(id),
    }
}

fn message_context_menu(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    origin: MessageOrigin,
    row: &ConnectionMessageRow<'_>,
    commands: &AppCommandSender,
) {
    set_menu_item_width(
        ui,
        &[
            "Put message into publish form",
            "Show in separate window",
            "Remove message",
            "Save message to cqm file",
            "Copy Topic to Clipboard",
            "Copy time to clipboard",
            "Copy payload to clipboard",
            "Clear list",
        ],
    );
    if menu_item(
        ui,
        Some(regular::UPLOAD_SIMPLE),
        "Put message into publish form",
    )
    .clicked()
    {
        send(commands, select_command(row.key));
        send(commands, copy_command(row.key));
        ui.close_menu();
    }
    if menu_item(
        ui,
        Some(regular::ARROW_SQUARE_OUT),
        "Show in separate window",
    )
    .clicked()
    {
        send(commands, select_command(row.key));
        open_message(ui, snapshot, row.key);
        ui.close_menu();
    }
    if menu_item(ui, Some(regular::TRASH), "Remove message").clicked() {
        send(commands, select_command(row.key));
        send(commands, remove_command(row.key));
        ui.close_menu();
    }
    if menu_item(
        ui,
        Some(regular::DOWNLOAD_SIMPLE),
        "Save message to cqm file",
    )
    .clicked()
    {
        send(commands, select_command(row.key));
        if let Some(path) = save_message_path(row.topic) {
            send(commands, export_command_to_path(row.key, path));
        }
        ui.close_menu();
    }
    ui.separator();
    if menu_item(ui, Some(regular::COPY), "Copy Topic to Clipboard").clicked() {
        send(commands, select_command(row.key));
        ui.ctx().copy_text(row.topic.to_owned());
        ui.close_menu();
    }
    if menu_item(ui, Some(regular::CLOCK), "Copy time to clipboard").clicked() {
        send(commands, select_command(row.key));
        ui.ctx().copy_text(row.timestamp.to_owned());
        ui.close_menu();
    }
    if menu_item(
        ui,
        Some(regular::CLIPBOARD_TEXT),
        "Copy payload to clipboard",
    )
    .clicked()
    {
        send(commands, select_command(row.key));
        if let Some(payload) = payload_text(snapshot, row.key) {
            ui.ctx().copy_text(payload);
        }
        ui.close_menu();
    }
    ui.separator();
    if menu_item(ui, Some(regular::BROOM), "Clear list").clicked() {
        send(commands, clear_command(origin));
        ui.close_menu();
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

fn payload_text(snapshot: &AppSnapshot, key: MessageKey) -> Option<String> {
    let payload = match key {
        MessageKey::Outgoing(id) => {
            &snapshot
                .workbench
                .publish
                .history
                .iter()
                .find(|row| row.id == id)?
                .payload
        }
        MessageKey::Incoming(id) => {
            &snapshot
                .workbench
                .messages
                .iter()
                .find(|message| message.id == id)?
                .payload
        }
    };
    Some(String::from_utf8_lossy(payload).into_owned())
}

fn open_message(ui: &Ui, snapshot: &AppSnapshot, key: MessageKey) {
    match key {
        MessageKey::Outgoing(id) => workbench_messages::open_outgoing_message(ui.ctx(), id),
        MessageKey::Incoming(id) => {
            if snapshot
                .workbench
                .messages
                .iter()
                .any(|message| message.id == id)
            {
                workbench_messages::open_incoming_message(ui.ctx(), id);
            }
        }
    }
}

fn selected_key(snapshot: &AppSnapshot, origin: MessageOrigin) -> Option<MessageKey> {
    match origin {
        MessageOrigin::Outgoing => snapshot
            .workbench
            .publish
            .selected_history_id
            .filter(|id| {
                snapshot
                    .workbench
                    .publish
                    .history
                    .iter()
                    .any(|row| row.id == *id)
            })
            .map(MessageKey::Outgoing),
        MessageOrigin::Incoming => snapshot
            .workbench
            .selected_message_id
            .filter(|id| {
                snapshot
                    .workbench
                    .messages
                    .iter()
                    .any(|message| message.id == *id)
            })
            .map(MessageKey::Incoming),
    }
}

fn source_has_messages(snapshot: &AppSnapshot, origin: MessageOrigin) -> bool {
    match origin {
        MessageOrigin::Outgoing => !snapshot.workbench.publish.history.is_empty(),
        MessageOrigin::Incoming => !snapshot.workbench.messages.is_empty(),
    }
}

fn auto_scroll_enabled(ui: &Ui, origin: MessageOrigin) -> bool {
    ui.ctx()
        .data_mut(|data| *data.get_persisted_mut_or(auto_scroll_id(origin), true))
}

fn set_auto_scroll_enabled(ui: &Ui, origin: MessageOrigin, enabled: bool) {
    ui.ctx()
        .data_mut(|data| data.insert_persisted(auto_scroll_id(origin), enabled));
}

fn auto_scroll_id(origin: MessageOrigin) -> Id {
    match origin {
        MessageOrigin::Outgoing => Id::new("outgoing-messages-auto-scroll"),
        MessageOrigin::Incoming => Id::new("incoming-messages-auto-scroll"),
    }
}
