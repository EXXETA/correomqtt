use correo_core::{AppCommand, AppCommandSender, AppSnapshot, ConnectionSurface, Workspace};
use correo_style::layout;
use egui::{CursorIcon, Id, Rect, RichText, Sense, Stroke, Ui, UiBuilder};

use crate::{
    about, broker, connection_launcher, connection_plugins, connection_settings, diagnostics,
    i18n::I18n, motion, plugins, responsive, scripts, settings, theme::ThemeTokens, widgets,
    widgets::paint_focus_outline, workbench, PayloadHighlighter,
};

const VIEW_PADDING: f32 = 10.0;
const VIEW_PADDING_TOP: f32 = 0.0;
const CONNECTION_DIVIDER_SIZE: f32 = 8.0;
const CONNECTION_DETAIL_MIN_WIDTH: f32 = 345.0;

pub fn sidebar(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    workspace: Workspace,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    ui.heading(i18n.workspace_label(workspace));
    ui.separator();
    match workspace {
        Workspace::ImportExport => transfer_sidebar(ui, snapshot, tokens, commands, i18n),
        Workspace::Scripts => scripts::sidebar(ui, &snapshot.scripts, tokens, commands, i18n),
        Workspace::Broker => {}
        Workspace::Plugins => {}
        Workspace::Diagnostics => {}
        Workspace::Settings => {}
        Workspace::About => {}
        Workspace::Connections => {}
    }
}

pub fn show(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
    payload_highlighter: Option<&PayloadHighlighter>,
    klingon_unlocked: bool,
    about_logo_triggered: &mut bool,
) {
    match snapshot.active_workspace {
        Workspace::Connections => {
            if responsive::connections_context_is_compact(ui.ctx(), snapshot.active_workspace) {
                padded_view_with_left_inset(ui, widgets::FLYOUT_HANDLE_WIDTH, |ui| {
                    connections(ui, snapshot, tokens, commands, i18n, payload_highlighter);
                });
            } else {
                padded_view(ui, |ui| {
                    connection_split(ui, snapshot, tokens, commands, i18n, payload_highlighter);
                });
            }
        }
        Workspace::ImportExport => {
            padded_view(ui, |ui| {
                workspace_title(ui, i18n.workspace_label(Workspace::ImportExport));
                import_export_launcher(ui, tokens, commands, i18n);
            });
        }
        Workspace::Scripts => scripts::show(ui, snapshot, tokens, commands, i18n),
        Workspace::Broker => padded_view(ui, |ui| {
            workspace_title(ui, i18n.workspace_label(Workspace::Broker));
            broker::show(ui, snapshot, tokens, commands, i18n);
        }),
        Workspace::Plugins => padded_view_with_left_inset(
            ui,
            if responsive::plugin_context_is_compact(ui.ctx()) {
                widgets::FLYOUT_HANDLE_WIDTH
            } else {
                0.0
            },
            |ui| {
                plugins::show(ui, snapshot, tokens, commands, i18n);
            },
        ),
        Workspace::Diagnostics => {
            padded_view(ui, |ui| {
                workspace_title(ui, i18n.workspace_label(Workspace::Diagnostics));
                diagnostics::workspace(ui, snapshot, tokens, i18n);
            });
        }
        Workspace::Settings => padded_view(ui, |ui| {
            settings::show(ui, snapshot, tokens, commands, i18n, klingon_unlocked);
        }),
        Workspace::About => {
            padded_view(ui, |ui| {
                workspace_title(ui, i18n.workspace_label(Workspace::About));
                *about_logo_triggered |= about::show(ui, tokens, i18n);
            });
        }
    }
}

fn padded_view(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) {
    padded_view_with_left_inset(ui, 0.0, add_contents);
}

fn padded_view_with_left_inset(
    ui: &mut Ui,
    extra_left_inset: f32,
    add_contents: impl FnOnce(&mut Ui),
) {
    let available = ui.available_rect_before_wrap();
    crate::overlay_bounds::set(ui, available);
    let rect = egui::Rect::from_min_max(
        egui::pos2(
            available.left() + VIEW_PADDING + extra_left_inset,
            available.top() + VIEW_PADDING_TOP,
        ),
        egui::pos2(
            available.right() - VIEW_PADDING,
            available.bottom() - VIEW_PADDING,
        ),
    );
    ui.allocate_rect(available, egui::Sense::hover());
    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(rect)
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );
    child.set_clip_rect(rect);
    add_contents(&mut child);
}

fn workspace_title(ui: &mut Ui, title: String) {
    ui.heading(title);
    ui.add_space(8.0);
}

fn connections(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
    payload_highlighter: Option<&PayloadHighlighter>,
) {
    match snapshot.connection_surface {
        ConnectionSurface::Launcher | ConnectionSurface::Workbench => {
            connection_workbench(ui, snapshot, tokens, commands, i18n, payload_highlighter)
        }
        ConnectionSurface::Settings if snapshot.selected_connection.is_none() => {
            connection_workbench(ui, snapshot, tokens, commands, i18n, payload_highlighter)
        }
        ConnectionSurface::Settings => {
            connection_settings::show(ui, snapshot, tokens, commands, i18n)
        }
        ConnectionSurface::Transfer => {
            connection_workbench(ui, snapshot, tokens, commands, i18n, payload_highlighter)
        }
    }
    if matches!(
        snapshot.connection_surface,
        ConnectionSurface::Launcher | ConnectionSurface::Workbench | ConnectionSurface::Settings
    ) {
        connection_settings::overlay(ui, snapshot, tokens, commands, i18n);
        connection_plugins::overlay(ui, snapshot, tokens, commands, i18n);
    }
}

fn connection_split(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
    payload_highlighter: Option<&PayloadHighlighter>,
) {
    let rect = ui.available_rect_before_wrap();
    ui.allocate_rect(rect, Sense::hover());
    let (left, right, divider) = connection_horizontal_split(ui, rect, tokens);
    connection_pane(ui, left, |ui| {
        connection_launcher::panel(ui, snapshot, tokens, commands, i18n);
    });
    connection_pane(ui, right, |ui| {
        connections(ui, snapshot, tokens, commands, i18n, payload_highlighter);
    });
    connection_flyout_mode_button(ui, divider, i18n);
}

fn connection_horizontal_split(ui: &mut Ui, rect: Rect, tokens: ThemeTokens) -> (Rect, Rect, Rect) {
    let usable = (rect.width() - CONNECTION_DIVIDER_SIZE).max(1.0);
    let min_left = layout::CONNECTION_SIDEBAR_MIN_WIDTH.min(usable * 0.45);
    let min_right = CONNECTION_DETAIL_MIN_WIDTH.min((usable - min_left).max(0.0));
    let max_left = (usable - min_right).max(min_left);
    let id = Id::new("connections-list-ratio");
    let default_ratio = (layout::CONNECTION_FLYOUT_WIDTH / usable).clamp(0.15, 0.85);
    let mut left_width = connection_ratio(ui, id, default_ratio) * usable;
    left_width = left_width.clamp(min_left, max_left);

    let divider = Rect::from_min_size(
        egui::pos2(rect.left() + left_width, rect.top()),
        egui::vec2(CONNECTION_DIVIDER_SIZE, rect.height()),
    );
    let response = ui
        .allocate_rect(divider, Sense::click_and_drag())
        .on_hover_cursor(CursorIcon::ResizeHorizontal);
    if response.dragged() {
        left_width = (left_width + response.drag_delta().x).clamp(min_left, max_left);
        store_connection_ratio(ui, id, left_width / usable);
    }
    draw_connection_divider(ui, divider, tokens, true);

    let left = Rect::from_min_max(
        rect.left_top(),
        egui::pos2(
            divider.left() - layout::WORKBENCH_CENTER_SPLIT_GUTTER,
            rect.bottom(),
        ),
    );
    let right = Rect::from_min_max(
        egui::pos2(
            divider.right() + layout::WORKBENCH_CENTER_SPLIT_GUTTER,
            rect.top(),
        ),
        rect.right_bottom(),
    );
    (left, right, divider)
}

fn connection_pane(ui: &mut Ui, rect: Rect, add_contents: impl FnOnce(&mut Ui)) {
    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(rect)
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );
    child.set_clip_rect(rect);
    add_contents(&mut child);
}

fn connection_flyout_mode_button(ui: &mut Ui, divider: Rect, i18n: &I18n) {
    if crate::widgets::flyout_mode_button_above_divider(
        ui,
        "connections-flyout-mode-button",
        divider,
        &i18n.text("connection-use-flyout"),
    )
    .clicked()
    {
        responsive::set_forced_context_flyout_mode(ui.ctx(), true);
        responsive::close_connection_flyout(ui.ctx());
        motion::finish_flyout_closed(ui.ctx(), "connections-context");
    }
}

fn draw_connection_divider(ui: &Ui, rect: Rect, tokens: ThemeTokens, vertical: bool) {
    let center = rect.center();
    let points = if vertical {
        [
            egui::pos2(center.x, rect.top()),
            egui::pos2(center.x, rect.bottom()),
        ]
    } else {
        [
            egui::pos2(rect.left(), center.y),
            egui::pos2(rect.right(), center.y),
        ]
    };
    ui.painter()
        .line_segment(points, Stroke::new(1.0, tokens.border));
}

fn connection_ratio(ui: &Ui, id: Id, default: f32) -> f32 {
    ui.ctx()
        .data_mut(|data| *data.get_persisted_mut_or(id, default))
        .clamp(0.15, 0.85)
}

fn store_connection_ratio(ui: &Ui, id: Id, value: f32) {
    ui.ctx()
        .data_mut(|data| data.insert_persisted(id, value.clamp(0.15, 0.85)));
}

fn import_export_launcher(
    ui: &mut Ui,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    ui.label(RichText::new(i18n.text("transfer-launch-detail")).color(tokens.text_secondary));
    ui.add_space(12.0);
    ui.horizontal(|ui| {
        let import = ui.button(i18n.text("transfer-import-title"));
        paint_focus_outline(ui, &import);
        if import.clicked() {
            send(commands, AppCommand::ImportConnections);
        }
        let export = ui.button(i18n.text("transfer-export-title"));
        paint_focus_outline(ui, &export);
        if export.clicked() {
            send(commands, AppCommand::ExportConnections);
        }
    });
}

fn connection_workbench(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
    payload_highlighter: Option<&PayloadHighlighter>,
) {
    if snapshot.selected_connection().is_some() {
        workbench::show(ui, snapshot, tokens, commands, i18n, payload_highlighter);
    } else {
        no_connection_available(ui, tokens, i18n);
    }
}

fn no_connection_available(ui: &mut Ui, tokens: ThemeTokens, i18n: &I18n) {
    ui.allocate_ui_with_layout(
        ui.available_size(),
        egui::Layout::centered_and_justified(egui::Direction::TopDown),
        |ui| {
            ui.label(
                RichText::new(i18n.text("connection-none-available"))
                    .size(16.0)
                    .color(tokens.text_secondary),
            );
        },
    );
}

fn transfer_sidebar(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    for section in [
        correo_core::TransferSection::Import,
        correo_core::TransferSection::Export,
    ] {
        let selected = snapshot.transfer.active_section == section;
        let label = match section {
            correo_core::TransferSection::Import => i18n.text("transfer-import-title"),
            correo_core::TransferSection::Export => i18n.text("transfer-export-title"),
            correo_core::TransferSection::Messages => section.label().to_owned(),
        };
        let response = ui.selectable_label(selected, label);
        paint_focus_outline(ui, &response);
        if response.clicked() {
            send(commands, AppCommand::SelectTransferSection(section));
        }
    }
    ui.separator();
    ui.label(
        RichText::new(format!(
            "{} import / {} export selected",
            snapshot.transfer.import.selected_count(),
            snapshot.transfer.export.selected_count()
        ))
        .color(tokens.text_secondary),
    );
    ui.separator();
    let import_cqc = ui.button(i18n.text("common-import-cqc"));
    paint_focus_outline(ui, &import_cqc);
    if import_cqc.clicked() {
        send(commands, AppCommand::ImportConnections);
    }
    let export_cqc = ui.button(i18n.text("common-export-cqc"));
    paint_focus_outline(ui, &export_cqc);
    if export_cqc.clicked() {
        send(commands, AppCommand::ExportConnections);
    }
}

fn send(commands: &AppCommandSender, command: AppCommand) {
    let _ = commands.send(command);
}
