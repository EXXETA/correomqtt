use correo_core::{
    AppCommand, AppCommandSender, AppSnapshot, ScriptExecutionRow, ScriptExecutionStatus,
    ScriptFileStatus, ScriptRow, ScriptSurfaceSnapshot,
};
use correo_style::layout as style_layout;
use egui::{Button, ComboBox, Id, Modal, RichText, ScrollArea, Sense, TextEdit, Ui};
use egui_phosphor::regular;

use crate::i18n::I18n;
use crate::modal_style;
use crate::motion;
use crate::payload_highlight;
use crate::responsive;
use crate::theme::{ThemeTokens, CONTROL_HEIGHT};
use crate::widgets::{
    clearable_search_edit, fill_remaining_tile_rows, padded_text_edit, square_icon_button_size,
    tile_list_content_width, tile_scroll_bar_rect_with_height, tile_table_fill,
    tile_table_hover_fill, tile_table_selected_fill, with_icon_button_padding, TILE_GAP,
    TWO_LINE_TILE_HEIGHT,
};

#[path = "scripts/dialogs.rs"]
mod dialogs;
#[path = "scripts/footer.rs"]
mod footer;
#[path = "scripts/layout.rs"]
mod layout;
#[path = "scripts/log.rs"]
mod log;

pub fn sidebar(
    ui: &mut Ui,
    scripts: &ScriptSurfaceSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    let mut filter = scripts.script_filter.clone();
    if clearable_search_edit(
        ui,
        None,
        &mut filter,
        "Search scripts...",
        tile_list_content_width(ui),
    )
    .changed()
    {
        send(commands, AppCommand::SearchScripts(filter));
    }
    ui.add_space(8.0);
    if ui
        .add_sized(
            [ui.available_width(), CONTROL_HEIGHT],
            Button::new("+ New Script"),
        )
        .clicked()
    {
        send(commands, AppCommand::RequestCreateScript);
    }
    ui.separator();
    let list_height = ui.available_height().max(style_layout::TABLE_MIN_HEIGHT);
    script_list(ui, scripts, tokens, commands, list_height);
}

pub fn show(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    if responsive::scripting_context_is_compact(ui.ctx()) {
        layout::right_panes(
            ui,
            tokens,
            |ui| {
                toolbar(ui, snapshot, commands);
                ui.add_space(6.0);
                editor(ui, &snapshot.scripts, commands);
            },
            |ui| log::log_view(ui, &snapshot.scripts, tokens, commands),
        );
        scripting_flyout(ui.ctx(), snapshot, tokens, commands, i18n);
    } else {
        layout::four_pane(
            ui,
            tokens,
            |ui| script_browser(ui, &snapshot.scripts, tokens, commands, i18n),
            |ui| {
                toolbar(ui, snapshot, commands);
                ui.add_space(6.0);
                editor(ui, &snapshot.scripts, commands);
            },
            |ui| executions(ui, &snapshot.scripts, tokens, commands),
            |ui| log::log_view(ui, &snapshot.scripts, tokens, commands),
        );
    }
    dialogs::create_dialog(ui, &snapshot.scripts, tokens, commands);
    rename_dialog(ui, &snapshot.scripts, tokens, commands);
    delete_dialog(ui, &snapshot.scripts, tokens, commands);
}

fn script_browser(
    ui: &mut Ui,
    scripts: &ScriptSurfaceSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    ui.allocate_ui_with_layout(
        egui::vec2(tile_list_content_width(ui), CONTROL_HEIGHT),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            if !responsive::forced_scripting_flyout_mode(ui.ctx())
                && !responsive::scripting_flyout_open(ui.ctx())
                && header_icon_button(ui, regular::LIST)
                    .on_hover_text("Use scripting flyout")
                    .clicked()
            {
                responsive::set_forced_scripting_flyout_mode(ui.ctx(), true);
                responsive::open_scripting_flyout(ui.ctx());
            }
            ui.heading(i18n.workspace_label(correo_core::Workspace::Scripts));
            ui.add_space(8.0);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if header_add_button(ui).on_hover_text("New Script").clicked() {
                    send(commands, AppCommand::RequestCreateScript);
                }
            });
        },
    );
    let mut filter = scripts.script_filter.clone();
    if clearable_search_edit(
        ui,
        None,
        &mut filter,
        "Search scripts...",
        tile_list_content_width(ui),
    )
    .changed()
    {
        send(commands, AppCommand::SearchScripts(filter));
    }
    ui.add_space(8.0);
    let list_height = ui.available_height().max(style_layout::TABLE_MIN_HEIGHT);
    ScrollArea::vertical()
        .id_salt("script-list")
        .max_height(list_height)
        .auto_shrink([false, false])
        .scroll_bar_rect(tile_scroll_bar_rect_with_height(ui, list_height))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            ui.set_width(tile_list_content_width(ui));
            script_list(ui, scripts, tokens, commands, list_height);
        });
}

fn header_add_button(ui: &mut Ui) -> egui::Response {
    header_icon_button(ui, regular::PLUS)
}

fn header_icon_button(ui: &mut Ui, icon: &'static str) -> egui::Response {
    with_icon_button_padding(ui, |ui| {
        ui.add_sized(
            square_icon_button_size(),
            Button::new(RichText::new(icon).size(15.0)),
        )
    })
}

fn script_list(
    ui: &mut Ui,
    scripts: &ScriptSurfaceSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    list_height: f32,
) {
    let filtered_scripts = scripts.filtered_scripts();
    let row_count = filtered_scripts.len();
    for (index, script) in filtered_scripts.into_iter().enumerate() {
        let selected = scripts.selected_script == script.name;
        let title = if script.is_dirty() {
            format!("{} *", script.name)
        } else {
            script.name.clone()
        };
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), TWO_LINE_TILE_HEIGHT),
            Sense::click(),
        );
        let fill = tile_fill(
            ui,
            ("script", script.name.as_str()),
            index,
            selected,
            response.hovered() || response.contains_pointer(),
            tokens,
        );
        ui.painter()
            .rect_filled(rect, egui::CornerRadius::ZERO, fill);
        paint_line(ui, rect, 0, &title, tokens.text_primary);
        let mut x = paint_line(
            ui,
            rect,
            1,
            script.status.label(),
            file_status_color(script.status, tokens),
        );
        x = paint_segment(
            ui,
            rect,
            1,
            x,
            &format!("{} runs", script.execution_count),
            tokens.text_secondary,
        );
        paint_segment(ui, rect, 1, x, &script.relative_path, tokens.text_secondary);
        response.context_menu(|ui| script_context_menu(ui, scripts, script, commands));
        if response.clicked() {
            send(commands, AppCommand::SelectScript(script.name.clone()));
            close_scripting_flyout_if_open(ui);
        }
        ui.add_space(TILE_GAP);
    }
    fill_remaining_tile_rows(ui, row_count, TWO_LINE_TILE_HEIGHT, list_height, tokens);
}

fn script_context_menu(
    ui: &mut Ui,
    scripts: &ScriptSurfaceSnapshot,
    script: &ScriptRow,
    commands: &AppCommandSender,
) {
    if ui
        .add_enabled(
            !scripts.running,
            Button::new(menu_label(regular::PLAY, "Run Script")),
        )
        .clicked()
    {
        send(commands, AppCommand::SelectScript(script.name.clone()));
        send(commands, AppCommand::RunScript);
        ui.close_menu();
    }
    ui.separator();
    if ui
        .button(menu_label(regular::PENCIL_SIMPLE, "Rename"))
        .clicked()
    {
        send(commands, AppCommand::SelectScript(script.name.clone()));
        send(commands, AppCommand::RequestRenameScript);
        ui.close_menu();
    }
    if ui.button(menu_label(regular::TRASH, "Delete...")).clicked() {
        send(commands, AppCommand::SelectScript(script.name.clone()));
        send(commands, AppCommand::RequestDeleteScript);
        ui.close_menu();
    }
}

fn paint_line(ui: &Ui, rect: egui::Rect, line: usize, text: &str, color: egui::Color32) -> f32 {
    paint_segment(ui, rect, line, rect.left() + 12.0, text, color)
}

fn tile_fill(
    ui: &Ui,
    id_source: impl std::hash::Hash,
    index: usize,
    selected: bool,
    hovered: bool,
    tokens: ThemeTokens,
) -> egui::Color32 {
    motion::tile_fill(
        ui,
        id_source,
        tile_table_fill(index, tokens),
        tile_table_hover_fill(tokens),
        tile_table_selected_fill(tokens),
        hovered,
        selected,
    )
}

fn paint_segment(
    ui: &Ui,
    rect: egui::Rect,
    line: usize,
    x: f32,
    text: &str,
    color: egui::Color32,
) -> f32 {
    let font = egui::TextStyle::Body.resolve(ui.style());
    let galley = ui.painter().layout_no_wrap(text.to_owned(), font, color);
    let pos = egui::pos2(x, rect.top() + 6.0 + (line as f32 * 20.0));
    ui.painter().galley(pos, galley.clone(), color);
    x + galley.size().x + 8.0
}

fn toolbar(ui: &mut Ui, snapshot: &AppSnapshot, commands: &AppCommandSender) {
    let scripts = &snapshot.scripts;
    let has_script = scripts.selected_script().is_some();
    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), CONTROL_HEIGHT),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            if responsive::scripting_context_is_compact(ui.ctx())
                && script_icon_button(ui, regular::LIST, true, "Show scripts and executions")
                    .clicked()
            {
                responsive::open_scripting_flyout(ui.ctx());
            }
            ui.heading(script_title(scripts));
            if script_icon_button(ui, regular::PENCIL_SIMPLE_LINE, has_script, "Rename script")
                .clicked()
            {
                send(commands, AppCommand::RequestRenameScript);
            }
            if script_icon_button(ui, regular::TRASH, has_script, "Delete script").clicked() {
                send(commands, AppCommand::RequestDeleteScript);
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                footer::help_link(ui);
            });
        },
    );
    ui.add_space(2.0);
    ui.horizontal(|ui| {
        if script_icon_button(ui, regular::FLOPPY_DISK, scripts.can_save(), "Save script").clicked()
        {
            send(commands, AppCommand::SaveScript);
        }
        if script_icon_button(
            ui,
            regular::ARROW_U_DOWN_LEFT,
            scripts.selected_script_is_dirty(),
            "Discard script changes",
        )
        .clicked()
        {
            send(commands, AppCommand::DiscardScriptChanges);
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .add_enabled(
                    scripts.can_run(),
                    Button::new(format!("{}  Run Script", regular::PLAY)),
                )
                .on_hover_text("Queue script execution through core")
                .clicked()
            {
                send(commands, AppCommand::RunScript);
            }
            ComboBox::from_id_salt("script-run-connection")
                .selected_text(&scripts.selected_connection)
                .width(220.0)
                .show_ui(ui, |ui| {
                    for connection in &snapshot.connections {
                        let id = connection.id.to_string();
                        let selected =
                            scripts.selected_connection_id.as_deref() == Some(id.as_str());
                        if ui.selectable_label(selected, &connection.name).clicked() {
                            send(commands, AppCommand::SelectScriptConnection(id));
                        }
                    }
                });
            ui.label("Run on");
        });
    });
}

fn script_title(scripts: &ScriptSurfaceSnapshot) -> String {
    if scripts.selected_script.is_empty() {
        "No script selected".to_owned()
    } else {
        scripts.selected_script.clone()
    }
}

fn script_icon_button(ui: &mut Ui, icon: &str, enabled: bool, hover_text: &str) -> egui::Response {
    let response = ui
        .add_enabled_ui(enabled, |ui| {
            with_icon_button_padding(ui, |ui| {
                ui.add_sized(
                    square_icon_button_size(),
                    Button::new(RichText::new(icon).size(16.0)),
                )
            })
        })
        .inner;
    response.on_hover_text(hover_text)
}

fn editor(ui: &mut Ui, scripts: &ScriptSurfaceSnapshot, commands: &AppCommandSender) {
    if let Some(script) = scripts.selected_script() {
        let mut source = script.source.clone();
        let editor_height = ui.available_height().max(180.0);
        let mut layouter = payload_highlight::javascript_layouter();
        if ui
            .add_sized(
                [ui.available_width(), editor_height],
                padded_text_edit(TextEdit::multiline(&mut source))
                    .font(egui::TextStyle::Monospace)
                    .desired_width(f32::INFINITY)
                    .layouter(&mut layouter),
            )
            .changed()
        {
            send(commands, AppCommand::UpdateScriptSource(source));
        }
    } else {
        ui.label("Select or create a script.");
    }
}

fn executions(
    ui: &mut Ui,
    scripts: &ScriptSurfaceSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    ui.horizontal(|ui| {
        ui.heading("Executions");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Clear execution log").clicked() {
                send(commands, AppCommand::ClearFinishedScriptExecutions);
            }
        });
    });
    ui.add_space(4.0);
    let list_height = ui.available_height().max(style_layout::TABLE_MIN_HEIGHT);
    ScrollArea::vertical()
        .id_salt("script-executions")
        .max_height(list_height)
        .auto_shrink([false, false])
        .scroll_bar_rect(tile_scroll_bar_rect_with_height(ui, list_height))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            ui.set_width(tile_list_content_width(ui));
            for (index, execution) in scripts.executions.iter().enumerate() {
                let selected =
                    scripts.selected_execution_id() == Some(execution.execution_id.as_str());
                let (rect, response) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), TWO_LINE_TILE_HEIGHT),
                    Sense::click(),
                );
                let fill = tile_fill(
                    ui,
                    ("execution", execution.execution_id.as_str()),
                    index,
                    selected,
                    response.hovered() || response.contains_pointer(),
                    tokens,
                );
                ui.painter()
                    .rect_filled(rect, egui::CornerRadius::ZERO, fill);
                let mut x = paint_line(
                    ui,
                    rect,
                    0,
                    execution.status.label(),
                    execution_color(execution.status, tokens),
                );
                x = paint_segment(ui, rect, 0, x, &execution.script_name, tokens.text_primary);
                paint_segment(ui, rect, 0, x, &execution.duration, tokens.text_secondary);
                let timestamp = crate::time_format::local_date_time(&execution.timestamp);
                paint_line(ui, rect, 1, &timestamp, tokens.text_secondary);
                response.context_menu(|ui| execution_context_menu(ui, execution, commands));
                if response.clicked() {
                    send(
                        commands,
                        AppCommand::SelectScriptExecution(execution.execution_id.clone()),
                    );
                    close_scripting_flyout_if_open(ui);
                }
                ui.add_space(TILE_GAP);
            }
            fill_remaining_tile_rows(
                ui,
                scripts.executions.len(),
                TWO_LINE_TILE_HEIGHT,
                list_height,
                tokens,
            );
        });
}

fn execution_context_menu(
    ui: &mut Ui,
    execution: &ScriptExecutionRow,
    commands: &AppCommandSender,
) {
    if ui
        .add_enabled(
            !execution.status.is_terminal(),
            Button::new(menu_label(regular::STOP, "Stop")),
        )
        .clicked()
    {
        send(
            commands,
            AppCommand::SelectScriptExecution(execution.execution_id.clone()),
        );
        send(commands, AppCommand::CancelScript);
        ui.close_menu();
    }
    if ui.button(menu_label(regular::TRASH, "Remove")).clicked() {
        send(
            commands,
            AppCommand::SelectScriptExecution(execution.execution_id.clone()),
        );
        send(
            commands,
            AppCommand::RemoveScriptExecution(execution.execution_id.clone()),
        );
        ui.close_menu();
    }
    ui.separator();
    if ui
        .button(menu_label(regular::BROOM, "Clear execution log"))
        .clicked()
    {
        send(commands, AppCommand::ClearFinishedScriptExecutions);
        ui.close_menu();
    }
}

fn scripting_flyout(
    ctx: &egui::Context,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    let open = responsive::scripting_flyout_open(ctx);
    let Some(progress) = motion::flyout_progress(ctx, "scripting-context", open) else {
        return;
    };
    if open && ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
        responsive::close_scripting_flyout(ctx);
    }

    let screen = ctx.screen_rect();
    let overlay_rect = egui::Rect::from_min_max(
        egui::pos2(
            screen.left() + style_layout::RAIL_WIDTH,
            screen.top() + style_layout::HEADER_HEIGHT,
        ),
        screen.right_bottom(),
    );
    if overlay_rect.width() <= 0.0 || overlay_rect.height() <= 0.0 {
        return;
    }

    egui::Area::new(egui::Id::new("scripting-context-flyout"))
        .order(egui::Order::Foreground)
        .fixed_pos(overlay_rect.min)
        .movable(false)
        .show(ctx, |ui| {
            let (scrim_rect, _) = ui.allocate_exact_size(overlay_rect.size(), Sense::hover());
            ui.painter().rect_filled(
                scrim_rect,
                egui::CornerRadius::ZERO,
                motion::scrim_color(modal_style::SCRIM_ALPHA, progress),
            );

            let panel_width = style_layout::SCRIPTING_FLYOUT_WIDTH.min(overlay_rect.width());
            let panel_rect = motion::flyout_panel_rect(scrim_rect, panel_width, progress);
            ui.painter()
                .rect_filled(panel_rect, egui::CornerRadius::ZERO, tokens.window_bg);

            let margin = style_layout::sidebar_margin();
            let content_rect = egui::Rect::from_min_max(
                egui::pos2(
                    panel_rect.left() + f32::from(margin.left),
                    panel_rect.top() + f32::from(margin.top),
                ),
                egui::pos2(
                    panel_rect.right() - f32::from(margin.right),
                    panel_rect.bottom() - f32::from(margin.bottom),
                ),
            );
            let mut panel_ui = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(content_rect)
                    .layout(egui::Layout::top_down(egui::Align::Min)),
            );
            panel_ui.multiply_opacity(motion::content_opacity(progress));
            panel_ui.set_clip_rect(content_rect);
            layout::list_column(
                &mut panel_ui,
                tokens,
                |ui| script_browser(ui, &snapshot.scripts, tokens, commands, i18n),
                |ui| executions(ui, &snapshot.scripts, tokens, commands),
            );
            scripting_flyout_restore_button(ui, panel_rect);

            let clicked_outside = open
                && ui.ctx().input(|input| {
                    input.pointer.any_click()
                        && input
                            .pointer
                            .interact_pos()
                            .is_some_and(|pos| !panel_rect.contains(pos))
                });
            if clicked_outside {
                responsive::close_scripting_flyout(ui.ctx());
            }
        });
}

fn scripting_flyout_restore_button(ui: &mut Ui, panel_rect: egui::Rect) {
    if !(responsive::forced_scripting_flyout_mode(ui.ctx())
        && !responsive::scripting_context_requires_flyout(ui.ctx()))
    {
        return;
    }

    let button_rect = egui::Rect::from_min_size(
        egui::pos2(
            panel_rect.right() + style_layout::TOOLBAR_GAP,
            panel_rect.top() + f32::from(style_layout::SIDEBAR_MARGIN_TOP),
        ),
        egui::Vec2::from(style_layout::square_icon_button_size()),
    );
    let mut button_ui = ui.new_child(egui::UiBuilder::new().max_rect(button_rect).layout(
        egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
    ));
    if button_ui
        .scope(|ui| {
            with_icon_button_padding(ui, |ui| {
                ui.add_sized(
                    style_layout::square_icon_button_size(),
                    Button::new(RichText::new(regular::SIDEBAR_SIMPLE).size(15.0)),
                )
            })
        })
        .inner
        .on_hover_text("Use scripting lists")
        .clicked()
    {
        responsive::set_forced_scripting_flyout_mode(ui.ctx(), false);
        responsive::close_scripting_flyout(ui.ctx());
    }
}

fn close_scripting_flyout_if_open(ui: &Ui) {
    if responsive::scripting_flyout_open(ui.ctx()) {
        responsive::close_scripting_flyout(ui.ctx());
    }
}

fn menu_label(icon: &str, label: &str) -> String {
    format!("{icon}  {label}")
}

fn rename_dialog(
    ui: &mut Ui,
    scripts: &ScriptSurfaceSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    if !scripts.rename_dialog_open {
        return;
    }
    let response = crate::modal_style::style(Modal::new(Id::new("rename-script-modal")), tokens)
        .show(ui.ctx(), |ui| {
            ui.set_width(360.0);
            ui.heading("Rename Script");
            let mut name = scripts.rename_script_name.clone();
            if ui
                .add_sized(
                    [ui.available_width(), CONTROL_HEIGHT],
                    padded_text_edit(TextEdit::singleline(&mut name)),
                )
                .changed()
            {
                send(commands, AppCommand::UpdateRenameScriptName(name));
            }
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), CONTROL_HEIGHT),
                egui::Layout::right_to_left(egui::Align::Center),
                |ui| {
                    if ui.button("Rename").clicked() {
                        send(commands, AppCommand::ConfirmRenameScript);
                    }
                    if ui.button("Cancel").clicked() {
                        send(commands, AppCommand::CancelRenameScript);
                    }
                },
            );
        });
    if response.should_close() {
        send(commands, AppCommand::CancelRenameScript);
    }
}

fn delete_dialog(
    ui: &mut Ui,
    scripts: &ScriptSurfaceSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    if !scripts.delete_confirmation_open {
        return;
    }
    let response = crate::modal_style::style(Modal::new(Id::new("delete-script-modal")), tokens)
        .show(ui.ctx(), |ui| {
            ui.set_width(360.0);
            ui.heading("Delete Script");
            ui.label(format!("Delete {}?", scripts.selected_script));
            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    send(commands, AppCommand::CancelDeleteScript);
                }
                if ui.button("Delete").clicked() {
                    send(commands, AppCommand::ConfirmDeleteScript);
                }
            });
        });
    if response.should_close() {
        send(commands, AppCommand::CancelDeleteScript);
    }
}

fn file_status_color(status: ScriptFileStatus, tokens: ThemeTokens) -> egui::Color32 {
    match status {
        ScriptFileStatus::Ready => tokens.success,
        ScriptFileStatus::Dirty => tokens.warning,
        ScriptFileStatus::Running => tokens.script,
        ScriptFileStatus::Error => tokens.danger,
    }
}

fn execution_color(status: ScriptExecutionStatus, tokens: ThemeTokens) -> egui::Color32 {
    match status {
        ScriptExecutionStatus::Queued | ScriptExecutionStatus::Running => tokens.script,
        ScriptExecutionStatus::Succeeded => tokens.success,
        ScriptExecutionStatus::Failed => tokens.danger,
        ScriptExecutionStatus::Cancelled => tokens.warning,
    }
}

fn send(commands: &AppCommandSender, command: AppCommand) {
    let _ = commands.send(command);
}
