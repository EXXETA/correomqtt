use correo_core::{
    AppCommand, AppCommandSender, AppSnapshot, ConnectionPluginDirection, ConnectionPluginWorkflow,
    ConnectionPluginWorkflowField, ConnectionPluginWorkflowStatus,
};
use egui::{
    Button, Color32, ComboBox, CornerRadius, Rect, RichText, Sense, TextEdit, Ui, UiBuilder,
};
use egui_phosphor::regular;

use crate::{
    i18n::I18n,
    theme::ThemeTokens,
    widgets::{
        checkbox, disable_tile_text_selection, fill_remaining_tile_rows, menu_item,
        padded_text_edit, set_text_menu_item_width, square_icon_button_size, tile_table_hover_fill,
        tile_table_interactive_fill, with_icon_button_padding,
    },
};

const SCRIM_ALPHA: u8 = 176;
const MODAL_RADIUS: u8 = 4;
const MODAL_PADDING: i8 = 12;
const DEFAULT_LEFT_PANE_WIDTH: f32 = 300.0;
const MIN_LEFT_PANE_WIDTH: f32 = 200.0;
const MIN_RIGHT_PANE_WIDTH: f32 = 300.0;
const DIVIDER_GAP: f32 = 26.0;
const DIVIDER_HIT_WIDTH: f32 = 10.0;
const FOOTER_HEIGHT: f32 = 54.0;
const SCREEN_MARGIN: f32 = 18.0;
const WORKFLOW_ROW_HEIGHT: f32 = 54.0;
const WORKFLOW_LIST_HEADER_HEIGHT: f32 = 42.0;
const DIVIDER_WIDTH_ID: &str = "connection-plugin-workflow-left-width";
const CONFIG_RIGHT_PADDING: f32 = 14.0;

pub(crate) fn overlay(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    if !snapshot.connection_settings.plugin_workflow_dialog_open {
        return;
    }
    if snapshot.connection_plugins_overlay.is_none()
        || snapshot.connection_plugins_overlay != snapshot.selected_connection
    {
        return;
    }
    if ui.ctx().input(|input| input.key_pressed(egui::Key::Escape)) {
        send(commands, AppCommand::CloseConnectionPlugins);
    }

    let overlay_rect = crate::overlay_bounds::get(ui);
    let safe_rect = overlay_rect.shrink(SCREEN_MARGIN);
    let modal_size = egui::vec2(
        (safe_rect.width() * 0.94).min(980.0),
        (safe_rect.height() * 0.9).min(720.0),
    );
    egui::Area::new(egui::Id::new("connection-plugin-workflows-overlay"))
        .order(egui::Order::Foreground)
        .fixed_pos(overlay_rect.min)
        .movable(false)
        .show(ui.ctx(), |ui| {
            let (scrim_rect, _) = ui.allocate_exact_size(overlay_rect.size(), Sense::click());
            ui.painter().rect_filled(
                scrim_rect,
                CornerRadius::ZERO,
                Color32::from_black_alpha(SCRIM_ALPHA),
            );
            let modal_rect = Rect::from_center_size(safe_rect.center(), modal_size);
            ui.painter().rect_filled(
                modal_rect,
                CornerRadius::same(MODAL_RADIUS),
                tokens.window_bg,
            );
            let content_rect = modal_rect.shrink(f32::from(MODAL_PADDING));
            let mut content = ui.new_child(UiBuilder::new().max_rect(content_rect));
            content.set_clip_rect(content_rect);
            dialog_content(&mut content, snapshot, tokens, commands, i18n);
        });
}

fn dialog_content(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    ui.horizontal(|ui| {
        ui.heading(i18n.text("validators-title"));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if icon_button(ui, regular::X, &i18n.text("common-cancel")).clicked() {
                send(commands, AppCommand::CloseConnectionPlugins);
            }
        });
    });
    ui.label(RichText::new(i18n.text("validators-detail")).color(tokens.text_secondary));
    ui.separator();

    let footer_top = ui.max_rect().bottom() - FOOTER_HEIGHT;
    let body_height = (footer_top - ui.cursor().min.y).max(180.0);
    let body_rect = Rect::from_min_size(
        ui.cursor().min,
        egui::vec2(ui.available_width(), body_height),
    );
    ui.allocate_rect(body_rect, Sense::hover());

    let left_width = divider_left_width(ui, body_rect.width());
    let left_rect = Rect::from_min_size(body_rect.min, egui::vec2(left_width, body_height));
    let right_left = left_rect.right() + DIVIDER_GAP;
    let right_rect = Rect::from_min_max(
        egui::pos2(right_left, body_rect.top()),
        body_rect.right_bottom(),
    );
    let separator_x = left_rect.right() + DIVIDER_GAP * 0.5;
    let divider_rect = Rect::from_center_size(
        egui::pos2(separator_x, body_rect.center().y),
        egui::vec2(DIVIDER_HIT_WIDTH, body_rect.height()),
    );
    let divider_response = ui
        .interact(
            divider_rect,
            ui.make_persistent_id("connection-plugin-workflow-divider"),
            Sense::click_and_drag(),
        )
        .on_hover_cursor(egui::CursorIcon::ResizeHorizontal);
    if divider_response.dragged() {
        if let Some(pointer) = divider_response.interact_pointer_pos() {
            let width = clamp_left_width(pointer.x - body_rect.left(), body_rect.width());
            ui.ctx()
                .data_mut(|data| data.insert_persisted(divider_width_id(), width));
        }
    }
    ui.painter().line_segment(
        [
            egui::pos2(separator_x, body_rect.top() + WORKFLOW_LIST_HEADER_HEIGHT),
            egui::pos2(separator_x, body_rect.bottom()),
        ],
        egui::Stroke::new(1.0, tokens.border),
    );

    let mut left_ui = ui.new_child(UiBuilder::new().max_rect(left_rect));
    left_ui.set_clip_rect(left_rect);
    workflow_list(&mut left_ui, snapshot, tokens, commands, body_height, i18n);

    let mut right_ui = ui.new_child(UiBuilder::new().max_rect(right_rect));
    right_ui.set_clip_rect(right_rect);
    selected_config(&mut right_ui, snapshot, tokens, commands, body_height, i18n);

    let footer_rect = Rect::from_min_max(
        egui::pos2(ui.max_rect().left(), footer_top + 8.0),
        ui.max_rect().right_bottom(),
    );
    let mut footer = ui.new_child(UiBuilder::new().max_rect(footer_rect));
    footer.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        if ui.button(i18n.text("common-cancel")).clicked() {
            send(commands, AppCommand::CloseConnectionPlugins);
        }
        if ui.button(i18n.text("common-save")).clicked() {
            send(commands, AppCommand::SaveConnectionPlugins);
        }
    });
}

fn divider_left_width(ui: &Ui, body_width: f32) -> f32 {
    let max_left = max_left_width(body_width);
    let stored = ui
        .ctx()
        .data_mut(|data| data.get_persisted::<f32>(divider_width_id()))
        .unwrap_or(DEFAULT_LEFT_PANE_WIDTH);
    stored.clamp(MIN_LEFT_PANE_WIDTH, max_left)
}

fn clamp_left_width(width: f32, body_width: f32) -> f32 {
    width.clamp(MIN_LEFT_PANE_WIDTH, max_left_width(body_width))
}

fn max_left_width(body_width: f32) -> f32 {
    (body_width - DIVIDER_GAP - MIN_RIGHT_PANE_WIDTH).max(MIN_LEFT_PANE_WIDTH)
}

fn divider_width_id() -> egui::Id {
    egui::Id::new(DIVIDER_WIDTH_ID)
}

fn workflow_list(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    height: f32,
    i18n: &I18n,
) {
    add_plugin_menu_button(ui, snapshot, commands, i18n);
    ui.add_space(6.0);
    let list_height = ui.available_height().max(120.0).min(height);
    egui::ScrollArea::vertical()
        .id_salt("connection-plugin-workflow-list-v2")
        .max_height(list_height)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            for (index, workflow) in snapshot
                .connection_settings
                .plugin_workflows
                .iter()
                .enumerate()
            {
                workflow_row(
                    ui,
                    index,
                    workflow,
                    tokens,
                    commands,
                    selected_workflow_index(snapshot) == Some(index),
                );
            }
            fill_remaining_tile_rows(
                ui,
                snapshot.connection_settings.plugin_workflows.len(),
                WORKFLOW_ROW_HEIGHT,
                list_height,
                tokens,
            );
        });
}

fn add_plugin_menu_button(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    let response = ui.button(format!("{} {}", regular::PLUS, i18n.text("validators-add")));
    let labels = available_workflow_plugin_labels(snapshot);
    let menu_width = crate::widgets::menu_item_content_width_without_icons(ui, &labels);
    left_aligned_menu(ui, response, menu_width, |ui| {
        add_plugin_menu(ui, snapshot, commands, i18n)
    });
}

fn left_aligned_menu(
    ui: &mut Ui,
    response: egui::Response,
    content_width: f32,
    add_contents: impl FnOnce(&mut Ui) -> bool,
) {
    let popup_id = response.id.with("left-aligned-menu");
    if response.clicked() {
        ui.ctx().data_mut(|data| {
            let open = data.get_temp::<bool>(popup_id).unwrap_or(false);
            data.insert_temp(popup_id, !open);
        });
    }
    let open = ui
        .ctx()
        .data_mut(|data| data.get_temp::<bool>(popup_id).unwrap_or(false));
    if !open {
        return;
    }

    let frame = egui::Frame::menu(ui.style());
    let menu_width = content_width + frame.total_margin().sum().x;
    let mut pos = response.rect.left_bottom();
    pos.y += ui.spacing().menu_spacing;
    if let Some(to_global) = ui.ctx().layer_transform_to_global(response.layer_id) {
        pos = to_global * pos;
    }
    let area_response = egui::Area::new(popup_id)
        .kind(egui::UiKind::Menu)
        .order(egui::Order::Foreground)
        .fixed_pos(pos)
        .default_width(menu_width)
        .show(ui.ctx(), |ui| frame.show(ui, add_contents).inner);

    let should_close = ui.ctx().input(|input| input.key_pressed(egui::Key::Escape))
        || (response.clicked_elsewhere() && area_response.response.clicked_elsewhere())
        || area_response.inner;
    if should_close {
        ui.ctx().data_mut(|data| data.insert_temp(popup_id, false));
    }
}

fn workflow_row(
    ui: &mut Ui,
    index: usize,
    workflow: &ConnectionPluginWorkflow,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    selected: bool,
) {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), WORKFLOW_ROW_HEIGHT),
        Sense::hover(),
    );
    let checkbox_rect = workflow_checkbox_rect(rect);
    let remove_rect = workflow_remove_rect(rect);
    let row_select_rect = Rect::from_min_max(
        egui::pos2(checkbox_rect.right(), rect.top()),
        egui::pos2(remove_rect.left(), rect.bottom()),
    );
    let response = ui.interact(
        row_select_rect,
        ui.make_persistent_id(("connection-plugin-workflow-row", index)),
        Sense::click_and_drag(),
    );
    response.dnd_set_drag_payload(index);
    let dragged = response.dragged();
    let drop_target =
        response.contains_pointer() && egui::DragAndDrop::has_any_payload(ui.ctx()) && !dragged;
    if let Some(dropped) = response.dnd_release_payload::<usize>() {
        if *dropped != index {
            let after = response
                .interact_pointer_pos()
                .or_else(|| ui.ctx().pointer_interact_pos())
                .is_some_and(|pointer| pointer.y > rect.center().y);
            send(
                commands,
                AppCommand::MoveConnectionPluginWorkflow {
                    index: *dropped,
                    target_index: index,
                    after,
                },
            );
        }
    }
    let fill = if dragged {
        tokens.panel_raised
    } else {
        tile_table_interactive_fill(index, tokens, response.hovered(), selected)
    };
    ui.painter().rect_filled(rect, CornerRadius::ZERO, fill);
    if dragged {
        ui.painter().rect_stroke(
            rect.shrink(2.0),
            CornerRadius::ZERO,
            egui::Stroke::new(2.0, tokens.accent),
            egui::StrokeKind::Inside,
        );
    }
    if drop_target {
        let after = ui
            .ctx()
            .pointer_interact_pos()
            .is_some_and(|pointer| pointer.y > rect.center().y);
        let y = if after {
            rect.bottom() - 2.0
        } else {
            rect.top() + 2.0
        };
        ui.painter().line_segment(
            [
                egui::pos2(rect.left() + 6.0, y),
                egui::pos2(rect.right() - 6.0, y),
            ],
            egui::Stroke::new(3.0, tokens.accent),
        );
    }
    if response.clicked() {
        send(commands, AppCommand::SelectConnectionPluginWorkflow(index));
    }
    workflow_row_contents(
        ui,
        rect,
        checkbox_rect,
        remove_rect,
        index,
        workflow,
        tokens,
        selected,
        commands,
    );
}

fn workflow_row_contents(
    ui: &mut Ui,
    rect: Rect,
    checkbox_rect: Rect,
    remove_rect: Rect,
    index: usize,
    workflow: &ConnectionPluginWorkflow,
    tokens: ThemeTokens,
    selected: bool,
    commands: &AppCommandSender,
) {
    let mut checkbox_ui = ui.new_child(UiBuilder::new().max_rect(checkbox_rect));
    if selected {
        let hover_fill = tile_table_hover_fill(tokens);
        checkbox_ui.visuals_mut().widgets.hovered.bg_fill = hover_fill;
        checkbox_ui.visuals_mut().widgets.active.bg_fill = hover_fill;
    }
    let mut enabled = workflow.enabled;
    if checkbox(&mut checkbox_ui, &mut enabled, "").changed() {
        send(
            commands,
            AppCommand::SetConnectionPluginWorkflowEnabled { index, enabled },
        );
    }

    if hover_icon_button(
        ui,
        remove_rect,
        index,
        regular::X,
        "Remove entry",
        selected,
        tokens,
    )
    .clicked()
    {
        send(
            commands,
            AppCommand::RemoveConnectionPluginWorkflow { index },
        );
    }

    let text_rect = Rect::from_min_max(
        egui::pos2(checkbox_rect.right() + 14.0, rect.top() + 8.0),
        egui::pos2(remove_rect.left() - 8.0, rect.bottom() - 6.0),
    );
    let mut text_ui = ui.new_child(UiBuilder::new().max_rect(text_rect));
    text_ui.set_clip_rect(text_rect);
    disable_tile_text_selection(&mut text_ui);
    text_ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 2.0;
        ui.add(egui::Label::new(RichText::new(&workflow.plugin_name).strong()).truncate());
        ui.label(
            RichText::new(format!(
                "{} · {}",
                workflow.kind.label(),
                workflow.status.label()
            ))
            .color(status_color(workflow.status, tokens)),
        );
    });
}

fn workflow_checkbox_rect(rect: Rect) -> Rect {
    Rect::from_center_size(
        egui::pos2(rect.left() + 20.0, rect.center().y - 3.0),
        egui::vec2(28.0, 28.0),
    )
}

fn workflow_remove_rect(rect: Rect) -> Rect {
    Rect::from_center_size(
        egui::pos2(rect.right() - 24.0, rect.center().y),
        egui::vec2(24.0, 24.0),
    )
}

fn hover_icon_button(
    ui: &mut Ui,
    rect: Rect,
    id_source: usize,
    icon: &str,
    tooltip: &str,
    selected: bool,
    tokens: ThemeTokens,
) -> egui::Response {
    let response = ui
        .interact(
            rect,
            ui.make_persistent_id(("hover-icon-button", tooltip, id_source)),
            Sense::click(),
        )
        .on_hover_text(tooltip)
        .on_hover_cursor(egui::CursorIcon::PointingHand);
    if response.hovered() {
        let fill = if selected {
            tile_table_hover_fill(tokens)
        } else {
            ui.visuals().widgets.hovered.bg_fill
        };
        ui.painter()
            .rect_filled(rect, ui.visuals().widgets.hovered.corner_radius, fill);
    }
    let color = if response.hovered() {
        ui.visuals().widgets.hovered.fg_stroke.color
    } else {
        ui.visuals().widgets.inactive.fg_stroke.color
    };
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        icon,
        egui::FontId::proportional(14.0),
        color,
    );
    response
}

fn available_workflow_plugin_labels(snapshot: &AppSnapshot) -> Vec<&str> {
    snapshot
        .plugins
        .plugins
        .iter()
        .filter(|plugin| {
            plugin.enabled
                && matches!(plugin.status, correo_core::PluginStatus::Active)
                && is_connection_workflow_plugin(&plugin.id)
        })
        .map(|plugin| plugin.name.as_str())
        .collect()
}

fn add_plugin_menu(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    commands: &AppCommandSender,
    i18n: &I18n,
) -> bool {
    let labels = available_workflow_plugin_labels(snapshot);
    set_text_menu_item_width(ui, &labels);
    let mut any = false;
    for plugin in snapshot.plugins.plugins.iter().filter(|plugin| {
        plugin.enabled
            && matches!(plugin.status, correo_core::PluginStatus::Active)
            && is_connection_workflow_plugin(&plugin.id)
    }) {
        any = true;
        if menu_item(ui, None, &plugin.name).clicked() {
            send(
                commands,
                AppCommand::AddConnectionPluginWorkflow {
                    plugin_id: plugin.id.clone(),
                },
            );
            return true;
        }
    }
    if !any {
        ui.label(i18n.text("validators-none-available"));
    }
    false
}

fn is_connection_workflow_plugin(plugin_id: &str) -> bool {
    matches!(
        plugin_id,
        "org.correomqtt.plugins.contains-string-validator"
            | "org.correomqtt.plugins.xml-xsd-validator"
            | "org.correomqtt.plugins.base64"
            | "org.correomqtt.plugins.save-manipulator"
            | "org.correomqtt.plugins.zip-manipulator"
    )
}

fn selected_config(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    height: f32,
    i18n: &I18n,
) {
    ui.add_space(WORKFLOW_LIST_HEADER_HEIGHT);
    if snapshot.connection_settings.plugin_workflows.is_empty() {
        ui.label(RichText::new(i18n.text("validators-empty-config")).color(tokens.text_secondary));
        return;
    };
    let Some(index) = selected_workflow_index(snapshot) else {
        return;
    };
    let Some(workflow) = snapshot.connection_settings.plugin_workflows.get(index) else {
        return;
    };
    ui.set_width(ui.available_width());
    egui::ScrollArea::vertical()
        .id_salt("connection-plugin-workflow-config")
        .max_height((height - WORKFLOW_LIST_HEADER_HEIGHT).max(120.0))
        .auto_shrink([false, false])
        .show(ui, |ui| {
            config_card(ui, index, workflow, tokens, commands, i18n)
        });
}

fn selected_workflow_index(snapshot: &AppSnapshot) -> Option<usize> {
    let len = snapshot.connection_settings.plugin_workflows.len();
    snapshot
        .connection_settings
        .selected_plugin_workflow
        .filter(|index| *index < len)
        .or_else(|| (len > 0).then_some(0))
}

fn config_card(
    ui: &mut Ui,
    index: usize,
    workflow: &ConnectionPluginWorkflow,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    config_form(ui, index, workflow, tokens, commands, i18n);
}

fn config_form(
    ui: &mut Ui,
    index: usize,
    workflow: &ConnectionPluginWorkflow,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    ui.heading(&workflow.plugin_name);
    if let Some(description) = workflow_configuration_description(&workflow.plugin_id) {
        ui.label(RichText::new(description).color(tokens.text_secondary));
        ui.add_space(8.0);
    }
    if !workflow.available {
        ui.label(RichText::new(i18n.text("validators-plugin-unavailable")).color(tokens.warning));
    }

    ui.label(i18n.text("validators-topic-filter"));
    text_field(
        ui,
        index,
        ConnectionPluginWorkflowField::TopicFilter,
        &workflow.topic_filter,
        commands,
    );

    ui.label(i18n.text("validators-direction"));
    ui.allocate_ui_with_layout(
        egui::vec2(180.0, crate::theme::CONTROL_HEIGHT),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ComboBox::from_id_salt(("connection-plugin-direction", index))
                .width(160.0)
                .selected_text(direction_label(i18n, workflow.direction))
                .show_ui(ui, |ui| {
                    for direction in ConnectionPluginDirection::ALL {
                        if ui
                            .selectable_label(
                                workflow.direction == direction,
                                direction_label(i18n, direction),
                            )
                            .clicked()
                        {
                            send(
                                commands,
                                AppCommand::SetConnectionPluginWorkflowDirection {
                                    index,
                                    direction,
                                },
                            );
                            ui.close_menu();
                        }
                    }
                });
        },
    );
    ui.add_space(8.0);

    match workflow.plugin_id.as_str() {
        "org.correomqtt.plugins.contains-string-validator" => {
            contains_config(ui, index, workflow, commands, i18n)
        }
        "org.correomqtt.plugins.xml-xsd-validator" => file_config(
            ui,
            index,
            workflow,
            &i18n.text("validators-xsd-file"),
            ConnectionPluginWorkflowField::XsdPath,
            commands,
            i18n,
        ),
        "org.correomqtt.plugins.save-manipulator" => {
            folder_config(ui, index, workflow, commands, i18n)
        }
        "org.correomqtt.plugins.base64" => {}
        "org.correomqtt.plugins.zip-manipulator" => {}
        _ => {
            ui.label(RichText::new(i18n.text("validators-no-config")).color(tokens.text_secondary));
        }
    };
}

fn direction_label(i18n: &I18n, direction: ConnectionPluginDirection) -> String {
    i18n.text(match direction {
        ConnectionPluginDirection::Incoming => "validators-direction-incoming",
        ConnectionPluginDirection::Outgoing => "validators-direction-outgoing",
        ConnectionPluginDirection::Both => "validators-direction-both",
    })
}

fn workflow_configuration_description(plugin_id: &str) -> Option<&'static str> {
    match plugin_id {
        "org.correomqtt.plugins.contains-string-validator" => Some("Checks text payloads for required literal or regex matches and marks messages that do not satisfy the rules."),
        "org.correomqtt.plugins.xml-xsd-validator" => Some("Validates XML payloads against the selected XSD schema and records validation errors on matching messages."),
        "org.correomqtt.plugins.base64" => Some("Encodes outgoing payloads as Base64 and decodes incoming Base64 payloads for matching topics."),
        "org.correomqtt.plugins.save-manipulator" => Some("Writes matching payloads to files in the selected folder without changing the message contents."),
        "org.correomqtt.plugins.zip-manipulator" => Some("Gzip-compresses outgoing payloads and decompresses incoming gzip payloads for matching topics."),
        _ => None,
    }
}

fn contains_config(
    ui: &mut Ui,
    index: usize,
    workflow: &ConnectionPluginWorkflow,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    ui.label(i18n.text("validators-strings-regex"));
    let value = workflow
        .config
        .get("rules")
        .and_then(serde_json::Value::as_array)
        .map(|rules| {
            rules
                .iter()
                .filter_map(|rule| {
                    let text = rule.get("text").and_then(serde_json::Value::as_str)?;
                    if rule
                        .get("regex")
                        .and_then(serde_json::Value::as_bool)
                        .unwrap_or(false)
                    {
                        Some(format!("regex:{text}"))
                    } else {
                        Some(text.to_owned())
                    }
                })
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default();
    multiline(
        ui,
        index,
        ConnectionPluginWorkflowField::ContainsStrings,
        &value,
        commands,
    );
}

fn file_config(
    ui: &mut Ui,
    index: usize,
    workflow: &ConnectionPluginWorkflow,
    label: &str,
    field: ConnectionPluginWorkflowField,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    ui.label(label);
    let value = workflow
        .config
        .get("xsd_path")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    let choose_label = i18n.text("validators-choose");
    let button_width = config_button_width(ui, &choose_label);
    let field_width =
        (ui.available_width() - button_width - ui.spacing().item_spacing.x - CONFIG_RIGHT_PADDING)
            .max(120.0);
    ui.horizontal(|ui| {
        text_field_width(ui, index, field, value, field_width, commands);
        if ui
            .add_sized(
                [button_width, crate::theme::CONTROL_HEIGHT],
                Button::new(choose_label),
            )
            .clicked()
        {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("XML Schema", &["xsd"])
                .pick_file()
            {
                send(
                    commands,
                    AppCommand::UpdateConnectionPluginWorkflowField {
                        index,
                        field,
                        value: path.to_string_lossy().into_owned(),
                    },
                );
            }
        }
    });
}

fn folder_config(
    ui: &mut Ui,
    index: usize,
    workflow: &ConnectionPluginWorkflow,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    ui.label(i18n.text("validators-save-folder"));
    let value = workflow
        .config
        .get("folder")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    let choose_label = i18n.text("validators-choose");
    let button_width = config_button_width(ui, &choose_label);
    let field_width =
        (ui.available_width() - button_width - ui.spacing().item_spacing.x - CONFIG_RIGHT_PADDING)
            .max(120.0);
    ui.horizontal(|ui| {
        text_field_width(
            ui,
            index,
            ConnectionPluginWorkflowField::SaveFolder,
            value,
            field_width,
            commands,
        );
        if ui
            .add_sized(
                [button_width, crate::theme::CONTROL_HEIGHT],
                Button::new(choose_label),
            )
            .clicked()
        {
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                send(
                    commands,
                    AppCommand::UpdateConnectionPluginWorkflowField {
                        index,
                        field: ConnectionPluginWorkflowField::SaveFolder,
                        value: path.to_string_lossy().into_owned(),
                    },
                );
            }
        }
    });
}

fn config_button_width(ui: &Ui, label: &str) -> f32 {
    let font = egui::TextStyle::Button.resolve(ui.style());
    let text_width = ui
        .painter()
        .layout_no_wrap(label.to_owned(), font, ui.visuals().text_color())
        .size()
        .x;
    text_width + ui.spacing().button_padding.x * 2.0
}

fn text_field(
    ui: &mut Ui,
    index: usize,
    field: ConnectionPluginWorkflowField,
    value: &str,
    commands: &AppCommandSender,
) {
    text_field_width(ui, index, field, value, ui.available_width(), commands);
}

fn text_field_width(
    ui: &mut Ui,
    index: usize,
    field: ConnectionPluginWorkflowField,
    value: &str,
    width: f32,
    commands: &AppCommandSender,
) {
    let mut value = value.to_owned();
    if ui
        .add(padded_text_edit(TextEdit::singleline(&mut value)).desired_width(width))
        .changed()
    {
        send(
            commands,
            AppCommand::UpdateConnectionPluginWorkflowField {
                index,
                field,
                value,
            },
        );
    }
}

fn multiline(
    ui: &mut Ui,
    index: usize,
    field: ConnectionPluginWorkflowField,
    value: &str,
    commands: &AppCommandSender,
) {
    let mut value = value.to_owned();
    if ui
        .add(
            padded_text_edit(TextEdit::multiline(&mut value))
                .desired_rows(8)
                .desired_width(f32::INFINITY),
        )
        .changed()
    {
        send(
            commands,
            AppCommand::UpdateConnectionPluginWorkflowField {
                index,
                field,
                value,
            },
        );
    }
}

fn icon_button(ui: &mut Ui, icon: &str, hover: &str) -> egui::Response {
    with_icon_button_padding(ui, |ui| {
        ui.add_sized(
            square_icon_button_size(),
            Button::new(RichText::new(icon).size(16.0)),
        )
    })
    .on_hover_text(hover)
}

fn status_color(status: ConnectionPluginWorkflowStatus, tokens: ThemeTokens) -> egui::Color32 {
    match status {
        ConnectionPluginWorkflowStatus::Ready | ConnectionPluginWorkflowStatus::Valid => {
            tokens.success
        }
        ConnectionPluginWorkflowStatus::Disabled => tokens.text_secondary,
        ConnectionPluginWorkflowStatus::MissingPlugin => tokens.warning,
        ConnectionPluginWorkflowStatus::Invalid | ConnectionPluginWorkflowStatus::Failed => {
            tokens.danger
        }
    }
}

fn send(commands: &AppCommandSender, command: AppCommand) {
    let _ = commands.send(command);
}
