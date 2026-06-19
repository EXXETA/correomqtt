use correo_core::{AppCommand, AppCommandSender, AppSnapshot};
use correo_style::layout;
use egui::{Button, Rect, RichText, Sense, TextEdit, Ui, UiBuilder};
use egui_phosphor::regular;

use crate::{
    payload_highlight, responsive,
    theme::{ThemeTokens, CONTROL_HEIGHT},
    widgets::{
        checkbox, edit_pulldown, padded_text_edit, paint_focus_outline, square_icon_button_size,
        with_icon_button_padding,
    },
    workbench_connection_messages::{self, MessageOrigin},
    workbench_helpers::{
        child_ui, connected, disconnected_action_button, qos_selector, right_rect, send,
        toolbar_rect,
    },
    workbench_layout::{self, WorkbenchPaneSide},
    PayloadHighlighter,
};

pub(crate) fn editor(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    payload_highlighter: Option<&PayloadHighlighter>,
) {
    workbench_layout::pane_title(ui, "Publish", WorkbenchPaneSide::Publish);
    ui.add_space(4.0);
    topic_row(ui, snapshot, tokens, commands);
    let mut payload = snapshot.workbench.publish.payload.clone();
    let remaining_rect = ui.available_rect_before_wrap();
    ui.allocate_rect(remaining_rect, Sense::hover());

    let retained_gap = 2.0;
    let retained_top = (remaining_rect.bottom() - CONTROL_HEIGHT).max(remaining_rect.top());
    let retained_row_rect = Rect::from_min_max(
        egui::pos2(remaining_rect.left(), retained_top),
        remaining_rect.right_bottom(),
    );
    let payload_bottom = (retained_row_rect.top() - retained_gap).max(remaining_rect.top());
    let payload_rect = Rect::from_min_max(
        remaining_rect.left_top(),
        egui::pos2(remaining_rect.right(), payload_bottom),
    );
    let mut layouter = payload_highlight::layouter(
        payload_highlighter.cloned(),
        snapshot.plugins.active_plugin_ids(),
    );
    let mut payload_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(payload_rect)
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );
    payload_ui.set_clip_rect(payload_rect);
    let payload_response = payload_ui.add_sized(
        [payload_rect.width(), payload_rect.height()],
        padded_text_edit(TextEdit::multiline(&mut payload))
            .font(egui::TextStyle::Monospace)
            .desired_width(f32::INFINITY)
            .layouter(&mut layouter),
    );
    if payload_response.changed() {
        send(commands, AppCommand::UpdatePublishPayload(payload));
    }
    retained_row(ui, retained_row_rect, snapshot, commands);
}

pub(crate) fn outgoing_messages(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    workbench_connection_messages::show(ui, snapshot, MessageOrigin::Outgoing, tokens, commands);
}

fn topic_row(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    let mut topic = snapshot.workbench.publish.topic.clone();
    let rect = toolbar_rect(ui);
    let icon_only = responsive::workbench_uses_icon_actions(rect.width());
    let is_connected = connected(snapshot);
    let can_publish = snapshot.workbench.publish.valid && is_connected;
    let action_width = if icon_only {
        square_icon_button_size()[0]
    } else {
        layout::PUBLISH_ACTION_BUTTON_WIDTH
    };

    let publish_rect = right_rect(rect, action_width, 0.0);
    let qos_rect = right_rect(rect, layout::QOS_WIDTH, action_width + layout::TOOLBAR_GAP);
    let folder_rect =
        Rect::from_min_size(rect.left_top(), egui::vec2(CONTROL_HEIGHT, CONTROL_HEIGHT));
    let topic_left = folder_rect.right() + layout::TOOLBAR_GAP;
    let topic_right = qos_rect.left() - layout::TOOLBAR_GAP;
    let topic_rect = Rect::from_min_max(
        egui::pos2(topic_left, rect.top()),
        egui::pos2(topic_right.max(topic_left), rect.bottom()),
    );

    child_ui(ui, folder_rect, |ui| {
        let open_file = with_icon_button_padding(ui, |ui| {
            ui.add_sized(
                square_icon_button_size(),
                Button::new(RichText::new(regular::FOLDER_OPEN).size(15.0)),
            )
        })
        .on_hover_text("Load a message file into the publish editor");
        paint_focus_outline(ui, &open_file);
        if open_file.clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("CorreoMQTT message", &["cqm"])
                .pick_file()
            {
                send(commands, AppCommand::ImportMessagesFromPath(path));
            }
        }
    });
    child_ui(ui, topic_rect, |ui| {
        let topic_response = edit_pulldown(
            ui,
            "publish-topic",
            &mut topic,
            "Topic",
            &snapshot.workbench.publish.topic_history,
            topic_rect.width(),
        );
        if topic_response.changed() {
            send(commands, AppCommand::UpdatePublishTopic(topic));
        }
    });
    child_ui(ui, qos_rect, |ui| {
        qos_selector(ui, "publish-qos", snapshot.workbench.publish.qos, |qos| {
            send(commands, AppCommand::UpdatePublishQos(qos));
        });
    });
    child_ui(ui, publish_rect, |ui| {
        let label = if icon_only {
            regular::PAPER_PLANE_RIGHT.to_owned()
        } else {
            format!("{}  Publish", regular::PAPER_PLANE_RIGHT)
        };
        if !is_connected {
            disconnected_action_button(
                ui,
                publish_rect.width(),
                label,
                "Publish is not available as long as the connection is not connected.",
                tokens,
            );
            return;
        }

        let publish = ui.add_enabled_ui(can_publish, |ui| {
            ui.spacing_mut().button_padding.x = 4.0;
            ui.add_sized([publish_rect.width(), CONTROL_HEIGHT], Button::new(&label))
        });
        let publish = publish.inner;
        paint_focus_outline(ui, &publish);
        if publish.clicked() {
            send(commands, AppCommand::Publish);
        }
        if !can_publish {
            publish.on_hover_text("Requires a valid topic.");
        } else if icon_only {
            publish.on_hover_text("Publish");
        }
    });
}

fn retained_row(
    ui: &mut Ui,
    retained_row_rect: Rect,
    snapshot: &AppSnapshot,
    commands: &AppCommandSender,
) {
    let retained_rect = right_rect(retained_row_rect, layout::RETAINED_WIDTH, 0.0);
    let mut retained_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(retained_rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    retained_ui.set_clip_rect(retained_rect);
    retained_ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        let mut retained = snapshot.workbench.publish.retained;
        if checkbox(ui, &mut retained, "Retained").changed() {
            send(commands, AppCommand::SetPublishRetained(retained));
        }
    });
}
