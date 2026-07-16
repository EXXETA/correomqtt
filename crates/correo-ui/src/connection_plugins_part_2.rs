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

