use correo_core::{
    AppCommand, AppCommandSender, AppSnapshot, PluginMetricListNode, PluginMetricRow, PluginUiNode,
    PluginWindowRow,
};
use egui::{CentralPanel, Context, Frame, Grid, Id, RichText, ScrollArea, ViewportBuilder};
use egui::{ViewportClass, ViewportId, Window};

use crate::theme::ThemeTokens;

pub fn show(
    context: &Context,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    for window in &snapshot.plugins.open_windows {
        if show_window(context, window, tokens) {
            let _ = commands.send(AppCommand::ClosePluginWindow {
                plugin_id: window.plugin_id.clone(),
                action_id: window.action_id.clone(),
                connection_id: window.connection_id,
            });
        }
    }
}

fn show_window(context: &Context, window: &PluginWindowRow, tokens: ThemeTokens) -> bool {
    let viewport_id = ViewportId::from_hash_of((
        "plugin-window-viewport",
        &window.plugin_id,
        &window.action_id,
        window.connection_id,
    ));
    let builder = ViewportBuilder::default()
        .with_title(&window.title)
        .with_inner_size([820.0, 520.0])
        .with_min_inner_size([460.0, 320.0]);

    context.show_viewport_immediate(viewport_id, builder, |ctx, class| {
        let close_requested = ctx.input(|input| input.viewport().close_requested());
        if class == ViewportClass::Embedded {
            let mut open = !close_requested;
            Window::new(&window.title)
                .id(Id::new((
                    "plugin-window",
                    &window.plugin_id,
                    &window.action_id,
                    window.connection_id,
                )))
                .open(&mut open)
                .default_size([820.0, 520.0])
                .show(ctx, |ui| render_window_contents(ui, window, tokens));
            close_requested || !open
        } else {
            CentralPanel::default()
                .frame(
                    Frame::NONE
                        .fill(tokens.window_bg)
                        .inner_margin(egui::Margin::same(correo_style::layout::CENTRAL_MARGIN)),
                )
                .show(ctx, |ui| render_window_contents(ui, window, tokens));
            close_requested
        }
    })
}

fn render_window_contents(ui: &mut egui::Ui, window: &PluginWindowRow, tokens: ThemeTokens) {
    for node in &window.nodes {
        render_node(ui, node, tokens);
    }
}

fn render_node(ui: &mut egui::Ui, node: &PluginUiNode, tokens: ThemeTokens) {
    match node {
        PluginUiNode::Heading { text } => {
            ui.heading(text);
        }
        PluginUiNode::Label { text } => {
            ui.label(text);
        }
        PluginUiNode::Separator => {
            ui.separator();
        }
        PluginUiNode::Table { columns, rows } => render_table(ui, columns, rows, tokens),
        PluginUiNode::MetricList(list) => render_metric_list(ui, list, tokens),
    }
}

fn render_metric_list(ui: &mut egui::Ui, list: &PluginMetricListNode, tokens: ThemeTokens) {
    ui.horizontal(|ui| {
        if ui.button("Copy to Clipboard").clicked() {
            ui.ctx().copy_text(list.copy_text.clone());
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.vertical(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    ui.label(RichText::new(&list.broker).strong());
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    ui.label(
                        RichText::new(&list.latest_update)
                            .size(12.0)
                            .color(tokens.text_secondary),
                    );
                });
            });
        });
    });
    ui.add_space(8.0);

    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show_rows(ui, 64.0, list.rows.len(), |ui, row_range| {
            for index in row_range {
                if let Some(row) = list.rows.get(index) {
                    render_metric_row(ui, row, index, tokens);
                }
            }
        });
}

fn render_metric_row(ui: &mut egui::Ui, row: &PluginMetricRow, index: usize, tokens: ThemeTokens) {
    let fill = if index % 2 == 0 {
        tokens.panel_bg
    } else {
        tokens.window_bg
    };
    let row_width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(row_width, 64.0), egui::Sense::hover());
    ui.painter()
        .rect_filled(rect, egui::CornerRadius::ZERO, fill);

    let left = rect.left() + 10.0;
    let right = rect.right() - 10.0;
    let name_font = egui::TextStyle::Button.resolve(ui.style());
    let description_font = egui::TextStyle::Small.resolve(ui.style());
    let value_font = egui::TextStyle::Button.resolve(ui.style());

    ui.painter().text(
        egui::pos2(left, rect.top() + 8.0),
        egui::Align2::LEFT_TOP,
        &row.name,
        name_font,
        tokens.text_primary,
    );
    ui.painter().text(
        egui::pos2(left, rect.top() + 31.0),
        egui::Align2::LEFT_TOP,
        &row.description,
        description_font,
        tokens.text_secondary,
    );
    ui.painter().text(
        egui::pos2(right, rect.top() + 8.0),
        egui::Align2::RIGHT_TOP,
        &row.value,
        value_font,
        tokens.text_primary,
    );
}

fn render_table(ui: &mut egui::Ui, columns: &[String], rows: &[Vec<String>], tokens: ThemeTokens) {
    Grid::new("plugin-window-table")
        .striped(true)
        .min_col_width(72.0)
        .show(ui, |ui| {
            for column in columns {
                ui.label(RichText::new(column).strong().color(tokens.text_primary));
            }
            ui.end_row();
            for row in rows {
                for index in 0..columns.len() {
                    ui.label(row.get(index).map(String::as_str).unwrap_or_default());
                }
                ui.end_row();
            }
        });
}
