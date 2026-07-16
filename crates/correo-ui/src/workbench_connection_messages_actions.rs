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
    metrics: MessageRowMetrics,
) {
    let row_width = ui.available_width();
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(row_width, metrics.height), Sense::CLICK);
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

    response.context_menu(|ui| message_context_menu(ui, snapshot, origin, row, commands, i18n));
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
    let topic_y = rect.top() + metrics.topic_y;
    let preview_y = rect.top() + metrics.preview_y;
    let topic_font = egui::TextStyle::Button.resolve(ui.style());
    let meta_font = egui::TextStyle::Small.resolve(ui.style());
    let timestamp = crate::time_format::local_date_time(row.timestamp);
    let size = formatted_size(row.byte_size);
    let qos_and_size = if row.retained {
        format!("{} · {} · {size}", i18n.text("message-retained"), row.qos)
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
        egui::pos2(meta_rect.right(), topic_y),
        &timestamp,
        ui.visuals().text_color(),
    );
    right_aligned_text(
        ui,
        egui::pos2(meta_rect.right(), preview_y),
        &qos_and_size,
        tokens.text_secondary,
    );
    if let Some((status, label)) = row.validation_status.zip(validation_label.as_deref()) {
        let dot_gap = text_width(ui, " · ", meta_font.clone());
        let x = meta_rect.right() - text_width(ui, &qos_and_size, meta_font.clone()) - dot_gap;
        right_aligned_text(
            ui,
            egui::pos2(x, preview_y),
            "·",
            tokens.text_secondary,
        );
        let x = x - dot_gap;
        right_aligned_text(
            ui,
            egui::pos2(x, preview_y),
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
    i18n: &I18n,
) {
    let copy_to_publish = i18n.text("message-action-copy-to-publish");
    let open_window = i18n.text("message-action-open-window");
    let remove = i18n.text("message-action-remove");
    let save = i18n.text("message-action-save");
    let copy_topic = i18n.text("message-action-copy-topic");
    let copy_time = i18n.text("message-action-copy-time");
    let copy_payload = i18n.text("message-action-copy-payload");
    let clear = i18n.text("message-action-clear");
    set_menu_item_width(
        ui,
        &[
            &copy_to_publish,
            &open_window,
            &remove,
            &save,
            &copy_topic,
            &copy_time,
            &copy_payload,
            &clear,
        ],
    );
    if menu_item(
        ui,
        Some(regular::UPLOAD_SIMPLE),
        &copy_to_publish,
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
        &open_window,
    )
    .clicked()
    {
        send(commands, select_command(row.key));
        open_message(ui, snapshot, row.key);
        ui.close_menu();
    }
    if menu_item(ui, Some(regular::TRASH), &remove).clicked() {
        send(commands, select_command(row.key));
        send(commands, remove_command(row.key));
        ui.close_menu();
    }
    if menu_item(
        ui,
        Some(regular::DOWNLOAD_SIMPLE),
        &save,
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
    if menu_item(ui, Some(regular::COPY), &copy_topic).clicked() {
        send(commands, select_command(row.key));
        ui.ctx().copy_text(row.topic.to_owned());
        ui.close_menu();
    }
    if menu_item(ui, Some(regular::CLOCK), &copy_time).clicked() {
        send(commands, select_command(row.key));
        ui.ctx().copy_text(row.timestamp.to_owned());
        ui.close_menu();
    }
    if menu_item(
        ui,
        Some(regular::CLIPBOARD_TEXT),
        &copy_payload,
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
    if menu_item(ui, Some(regular::BROOM), &clear).clicked() {
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

