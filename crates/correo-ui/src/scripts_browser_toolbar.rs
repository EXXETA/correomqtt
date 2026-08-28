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
