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
    tile_table_hover_fill, tile_table_selected_fill, with_icon_button_padding, FLYOUT_HANDLE_WIDTH,
    TILE_GAP, TWO_LINE_TILE_HEIGHT,
};

#[path = "scripts/dialogs.rs"]
mod dialogs;
#[path = "scripts/footer.rs"]
mod footer;
#[path = "scripts/layout.rs"]
mod layout;
#[path = "scripts/log.rs"]
mod log;

const SCRIPT_CONNECTION_COMBO_MAX_WIDTH: f32 = 220.0;
const SCRIPT_CONNECTION_COMBO_MIN_WIDTH: f32 = 96.0;
const SCRIPT_RUN_BUTTON_WIDTH: f32 = 128.0;

pub fn sidebar(
    ui: &mut Ui,
    scripts: &ScriptSurfaceSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    let mut filter = scripts.script_filter.clone();
    if clearable_search_edit(
        ui,
        None,
        &mut filter,
        i18n.text("script-search"),
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
            Button::new(format!("+ {}", i18n.text("script-new"))),
        )
        .clicked()
    {
        send(commands, AppCommand::RequestCreateScript);
    }
    ui.separator();
    let list_height = ui.available_height().max(style_layout::TABLE_MIN_HEIGHT);
    script_list(ui, scripts, tokens, commands, i18n, list_height);
}

pub fn show(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    if responsive::scripting_context_is_compact(ui.ctx()) {
        compact_right_panes(ui, snapshot, tokens, commands, i18n);
        scripting_flyout(ui.ctx(), snapshot, tokens, commands, i18n);
    } else {
        layout::four_pane(
            ui,
            tokens,
            |ui| script_browser(ui, &snapshot.scripts, tokens, commands, i18n),
            |ui| {
                toolbar(ui, snapshot, commands, i18n);
                ui.add_space(6.0);
                editor(ui, &snapshot.scripts, commands, i18n);
            },
            |ui| executions(ui, &snapshot.scripts, tokens, commands, i18n),
            |ui| log::log_view(ui, &snapshot.scripts, tokens, commands, i18n),
            |ui, divider| scripting_flyout_mode_button(ui, divider, i18n),
        );
    }
    dialogs::create_dialog(ui, &snapshot.scripts, tokens, commands, i18n);
    rename_dialog(ui, &snapshot.scripts, tokens, commands, i18n);
    delete_dialog(ui, &snapshot.scripts, tokens, commands, i18n);
}

fn compact_right_panes(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    let available = ui.available_rect_before_wrap();
    ui.allocate_rect(available, Sense::hover());
    let rect = egui::Rect::from_min_max(
        egui::pos2(available.left() + FLYOUT_HANDLE_WIDTH, available.top()),
        available.right_bottom(),
    );
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect)
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );
    child.set_clip_rect(rect);
    layout::right_panes(
        &mut child,
        tokens,
        |ui| {
            toolbar(ui, snapshot, commands, i18n);
            ui.add_space(6.0);
            editor(ui, &snapshot.scripts, commands, i18n);
        },
        |ui| log::log_view(ui, &snapshot.scripts, tokens, commands, i18n),
    );
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
            ui.heading(i18n.workspace_label(correo_core::Workspace::Scripts));
            ui.add_space(8.0);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if header_add_button(ui)
                    .on_hover_text(i18n.text("script-new"))
                    .clicked()
                {
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
        i18n.text("script-search"),
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
            script_list(ui, scripts, tokens, commands, i18n, list_height);
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
    i18n: &I18n,
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
            &i18n.text_with_args(
                "script-runs",
                &[("count", script.execution_count.to_string())],
            ),
            tokens.text_secondary,
        );
        paint_segment(ui, rect, 1, x, &script.relative_path, tokens.text_secondary);
        response.context_menu(|ui| script_context_menu(ui, scripts, script, commands, i18n));
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
    i18n: &I18n,
) {
    if ui
        .add_enabled(
            !scripts.running,
            Button::new(menu_label(regular::PLAY, &i18n.text("script-run"))),
        )
        .clicked()
    {
        send(commands, AppCommand::SelectScript(script.name.clone()));
        send(commands, AppCommand::RunScript);
        ui.close_menu();
    }
    ui.separator();
    if ui
        .button(menu_label(
            regular::PENCIL_SIMPLE,
            &i18n.text("script-rename"),
        ))
        .clicked()
    {
        send(commands, AppCommand::SelectScript(script.name.clone()));
        send(commands, AppCommand::RequestRenameScript);
        ui.close_menu();
    }
    if ui
        .button(menu_label(
            regular::TRASH,
            &format!("{}...", i18n.text("common-delete")),
        ))
        .clicked()
    {
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

fn toolbar(ui: &mut Ui, snapshot: &AppSnapshot, commands: &AppCommandSender, i18n: &I18n) {
    let scripts = &snapshot.scripts;
    let has_script = scripts.selected_script().is_some();
    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), CONTROL_HEIGHT),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.heading(script_title(scripts, i18n));
            if script_icon_button(
                ui,
                regular::PENCIL_SIMPLE_LINE,
                has_script,
                &i18n.text("script-rename-tooltip"),
            )
            .clicked()
            {
                send(commands, AppCommand::RequestRenameScript);
            }
            if script_icon_button(
                ui,
                regular::TRASH,
                has_script,
                &i18n.text("script-delete-tooltip"),
            )
            .clicked()
            {
                send(commands, AppCommand::RequestDeleteScript);
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                footer::help_link(ui);
            });
        },
    );
    ui.add_space(2.0);
    ui.horizontal(|ui| {
        if script_icon_button(
            ui,
            regular::FLOPPY_DISK,
            scripts.can_save(),
            &i18n.text("script-save"),
        )
        .clicked()
        {
            send(commands, AppCommand::SaveScript);
        }
        if script_icon_button(
            ui,
            regular::ARROW_U_DOWN_LEFT,
            scripts.selected_script_is_dirty(),
            &i18n.text("script-discard"),
        )
        .clicked()
        {
            send(commands, AppCommand::DiscardScriptChanges);
        }
        let run_toolbar_width = ui.available_width();
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let gap = ui.spacing().item_spacing.x;
            let run_icon_only = run_toolbar_width < 460.0;
            let run_width = if run_icon_only {
                square_icon_button_size()[0]
            } else {
                SCRIPT_RUN_BUTTON_WIDTH
            };
            let run_on_width = if run_toolbar_width < 330.0 {
                0.0
            } else {
                text_width(ui, &i18n.text("script-run-on"))
            };
            let combo_width = (run_toolbar_width - run_width - run_on_width - gap * 3.0).clamp(
                SCRIPT_CONNECTION_COMBO_MIN_WIDTH,
                SCRIPT_CONNECTION_COMBO_MAX_WIDTH,
            );
            let selected_connection = ellipsize_to_width(
                ui,
                &scripts.selected_connection,
                (combo_width - 28.0).max(24.0),
            );
            if ui
                .add_enabled(
                    scripts.can_run(),
                    Button::new(if run_icon_only {
                        regular::PLAY.to_owned()
                    } else {
                        format!("{}  {}", regular::PLAY, i18n.text("script-run"))
                    })
                    .min_size(egui::vec2(run_width, CONTROL_HEIGHT)),
                )
                .on_hover_text(i18n.text("script-run-tooltip"))
                .clicked()
            {
                send(commands, AppCommand::RunScript);
            }
            ComboBox::from_id_salt("script-run-connection")
                .selected_text(selected_connection)
                .width(combo_width)
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
            if run_on_width > 0.0 {
                ui.label(i18n.text("script-run-on"));
            }
        });
    });
}

fn text_width(ui: &Ui, text: &str) -> f32 {
    let font = egui::TextStyle::Body.resolve(ui.style());
    ui.painter()
        .layout_no_wrap(text.to_owned(), font, ui.visuals().text_color())
        .size()
        .x
}

fn ellipsize_to_width(ui: &Ui, text: &str, max_width: f32) -> String {
    if text_width(ui, text) <= max_width {
        return text.to_owned();
    }

    let ellipsis = "...";
    let mut output = String::new();
    for ch in text.chars() {
        let candidate = format!("{output}{ch}{ellipsis}");
        if text_width(ui, &candidate) > max_width {
            break;
        }
        output.push(ch);
    }
    if output.is_empty() {
        ellipsis.to_owned()
    } else {
        format!("{output}{ellipsis}")
    }
}

fn script_title(scripts: &ScriptSurfaceSnapshot, i18n: &I18n) -> String {
    if scripts.selected_script.is_empty() {
        i18n.text("script-none-selected")
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

fn editor(ui: &mut Ui, scripts: &ScriptSurfaceSnapshot, commands: &AppCommandSender, i18n: &I18n) {
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
        ui.label(i18n.text("script-select-or-create"));
    }
}

fn executions(
    ui: &mut Ui,
    scripts: &ScriptSurfaceSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    ui.horizontal(|ui| {
        ui.heading(i18n.text("script-executions"));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button(i18n.text("script-clear-execution-log")).clicked() {
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
                response.context_menu(|ui| execution_context_menu(ui, execution, commands, i18n));
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
    i18n: &I18n,
) {
    if ui
        .add_enabled(
            !execution.status.is_terminal(),
            Button::new(menu_label(regular::STOP, &i18n.text("script-stop-short"))),
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
    if ui
        .button(menu_label(
            regular::TRASH,
            &i18n.text("script-remove-execution"),
        ))
        .clicked()
    {
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
        .button(menu_label(
            regular::BROOM,
            &i18n.text("script-clear-execution-log"),
        ))
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

    let open = responsive::scripting_flyout_open(ctx);
    if !open {
        scripting_flyout_collapsed_handle(ctx, overlay_rect, i18n);
    }
    let Some(progress) = motion::flyout_progress(ctx, "scripting-context", open) else {
        return;
    };
    if open && ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
        responsive::close_scripting_flyout(ctx);
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

            let handle_rect = egui::Rect::from_min_size(
                scrim_rect.left_top(),
                egui::vec2(FLYOUT_HANDLE_WIDTH, scrim_rect.height()),
            );
            let panel_area = egui::Rect::from_min_max(
                egui::pos2(handle_rect.right(), scrim_rect.top()),
                scrim_rect.right_bottom(),
            );
            let panel_width = style_layout::SCRIPTING_FLYOUT_WIDTH.min(panel_area.width());
            let panel_rect = motion::flyout_panel_rect(panel_area, panel_width, progress);
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
                |ui| executions(ui, &snapshot.scripts, tokens, commands, i18n),
            );
            scripting_flyout_expanded_controls(ui, handle_rect, panel_rect, i18n);

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

fn scripting_flyout_collapsed_handle(ctx: &egui::Context, overlay_rect: egui::Rect, i18n: &I18n) {
    egui::Area::new(egui::Id::new("scripting-context-flyout-collapsed-handle"))
        .order(egui::Order::Foreground)
        .fixed_pos(overlay_rect.min)
        .movable(false)
        .show(ctx, |ui| {
            let handle_rect = egui::Rect::from_min_size(
                ui.min_rect().min,
                egui::vec2(FLYOUT_HANDLE_WIDTH, overlay_rect.height()),
            );
            if crate::widgets::flyout_handle(
                ui,
                handle_rect,
                "scripting-flyout-open-handle",
                regular::CARET_RIGHT,
                &i18n.text("script-show-list"),
            )
            .clicked()
            {
                responsive::open_scripting_flyout(ui.ctx());
            }
        });
}

fn scripting_flyout_mode_button(ui: &mut Ui, divider: egui::Rect, i18n: &I18n) {
    if crate::widgets::flyout_mode_button_above_divider(
        ui,
        "scripting-flyout-mode-button",
        divider,
        &i18n.text("script-use-flyout"),
    )
    .clicked()
    {
        responsive::set_forced_context_flyout_mode(ui.ctx(), true);
        responsive::close_scripting_flyout(ui.ctx());
        motion::finish_flyout_closed(ui.ctx(), "scripting-context");
    }
}

fn scripting_flyout_expanded_controls(
    ui: &mut Ui,
    handle_rect: egui::Rect,
    panel_rect: egui::Rect,
    i18n: &I18n,
) {
    if crate::widgets::flyout_handle(
        ui,
        handle_rect,
        "scripting-flyout-collapse-handle",
        regular::CARET_LEFT,
        &i18n.text("script-collapse-flyout"),
    )
    .clicked()
    {
        responsive::close_scripting_flyout(ui.ctx());
    }

    if !responsive::forced_scripting_flyout_mode(ui.ctx())
        || responsive::scripting_context_requires_flyout(ui.ctx())
    {
        return;
    }

    if crate::widgets::flyout_restore_button_above_edge(
        ui,
        "scripting-flyout-restore-button",
        panel_rect.right(),
        panel_rect.top(),
        &i18n.text("script-use-lists"),
    )
    .clicked()
    {
        responsive::set_forced_context_flyout_mode(ui.ctx(), false);
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
    i18n: &I18n,
) {
    if !scripts.rename_dialog_open {
        return;
    }
    let response = crate::modal_style::style(Modal::new(Id::new("rename-script-modal")), tokens)
        .show(ui.ctx(), |ui| {
            ui.set_width(360.0);
            ui.heading(i18n.text("script-rename-tooltip"));
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
                    if ui.button(i18n.text("script-rename")).clicked() {
                        send(commands, AppCommand::ConfirmRenameScript);
                    }
                    if ui.button(i18n.text("common-cancel")).clicked() {
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
    i18n: &I18n,
) {
    if !scripts.delete_confirmation_open {
        return;
    }
    let response = crate::modal_style::style(Modal::new(Id::new("delete-script-modal")), tokens)
        .show(ui.ctx(), |ui| {
            ui.set_width(360.0);
            ui.heading(i18n.text("script-delete"));
            ui.label(i18n.text_with_args(
                "script-delete-detail",
                &[("name", scripts.selected_script.clone())],
            ));
            ui.horizontal(|ui| {
                if ui.button(i18n.text("common-cancel")).clicked() {
                    send(commands, AppCommand::CancelDeleteScript);
                }
                if ui.button(i18n.text("common-delete")).clicked() {
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
