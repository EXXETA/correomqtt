use correo_core::{AppCommand, AppCommandSender, AppSnapshot, ConnectionSurface, Workspace};
use egui::{RichText, Ui, UiBuilder};

use crate::{
    about, connection_plugins, connection_settings, diagnostics, i18n::I18n, plugins, responsive,
    scripts, settings, theme::ThemeTokens, widgets, widgets::paint_focus_outline, workbench,
    PayloadHighlighter,
};

const VIEW_PADDING: f32 = 10.0;
const VIEW_PADDING_TOP: f32 = 0.0;

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
        Workspace::Connections => padded_view_with_left_inset(
            ui,
            if responsive::connections_context_is_compact(ui.ctx(), snapshot.active_workspace) {
                widgets::FLYOUT_HANDLE_WIDTH
            } else {
                0.0
            },
            |ui| {
                connections(ui, snapshot, tokens, commands, i18n, payload_highlighter);
            },
        ),
        Workspace::ImportExport => {
            padded_view(ui, |ui| {
                workspace_title(ui, i18n.workspace_label(Workspace::ImportExport));
                import_export_launcher(ui, tokens, commands, i18n);
            });
        }
        Workspace::Scripts => scripts::show(ui, snapshot, tokens, commands, i18n),
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
