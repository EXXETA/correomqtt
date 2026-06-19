use correo_core::{sample_snapshot, AppCommandSender, AppSnapshot, ThemeMode, Workspace};
use correo_style::{apply_theme, layout, tokens, ThemeTokens};
use egui::{CentralPanel, Frame, SidePanel, TopBottomPanel};
use egui_phosphor::regular;

use crate::{
    command_bar, connection_launcher, i18n::I18n, icons, migration_recovery, motion, nav,
    responsive, toasts, transfer_wizard, widgets, workspace, PayloadHighlighter,
};

pub const THEME_KEY: &str = "correo.theme-mode";

pub struct CorreoUi {
    command_sender: AppCommandSender,
    snapshot: AppSnapshot,
    i18n: I18n,
    icons_installed: bool,
    transfer_wizard: transfer_wizard::State,
    payload_highlighter: Option<PayloadHighlighter>,
    klingon_easter_egg_clicks: u8,
}

impl CorreoUi {
    pub fn new(creation_context: &eframe::CreationContext<'_>) -> Self {
        let theme_mode = stored_theme(creation_context);
        let snapshot = sample_snapshot(theme_mode.clone());
        apply_theme(&creation_context.egui_ctx, &theme_mode);
        egui_extras::install_image_loaders(&creation_context.egui_ctx);
        icons::install(&creation_context.egui_ctx);
        Self {
            command_sender: AppCommandSender::disconnected(),
            i18n: I18n::new(&snapshot.global_settings.language),
            snapshot,
            icons_installed: true,
            transfer_wizard: transfer_wizard::State::default(),
            payload_highlighter: None,
            klingon_easter_egg_clicks: 0,
        }
    }

    pub fn for_snapshot(snapshot: AppSnapshot) -> Self {
        Self {
            command_sender: AppCommandSender::disconnected(),
            i18n: I18n::new(&snapshot.global_settings.language),
            snapshot,
            icons_installed: false,
            transfer_wizard: transfer_wizard::State::default(),
            payload_highlighter: None,
            klingon_easter_egg_clicks: 0,
        }
    }

    pub fn for_snapshot_with_command_sender(
        snapshot: AppSnapshot,
        command_sender: AppCommandSender,
    ) -> Self {
        Self {
            command_sender,
            i18n: I18n::new(&snapshot.global_settings.language),
            snapshot,
            icons_installed: false,
            transfer_wizard: transfer_wizard::State::default(),
            payload_highlighter: None,
            klingon_easter_egg_clicks: 0,
        }
    }

    pub fn with_command_sender(
        creation_context: &eframe::CreationContext<'_>,
        snapshot: AppSnapshot,
        command_sender: AppCommandSender,
        payload_highlighter: Option<PayloadHighlighter>,
    ) -> Self {
        apply_theme(&creation_context.egui_ctx, &snapshot.theme_mode);
        egui_extras::install_image_loaders(&creation_context.egui_ctx);
        icons::install(&creation_context.egui_ctx);
        Self {
            command_sender,
            i18n: I18n::new(&snapshot.global_settings.language),
            snapshot,
            icons_installed: true,
            transfer_wizard: transfer_wizard::State::default(),
            payload_highlighter,
            klingon_easter_egg_clicks: 0,
        }
    }

    pub fn for_theme(theme_mode: ThemeMode) -> Self {
        Self::for_snapshot(sample_snapshot(theme_mode))
    }

    pub fn set_snapshot(&mut self, snapshot: AppSnapshot) {
        self.i18n.set_language(&snapshot.global_settings.language);
        self.snapshot = snapshot;
    }

    pub fn theme_mode(&self) -> ThemeMode {
        self.snapshot.theme_mode.clone()
    }

    pub fn draw(&mut self, context: &egui::Context) {
        self.ensure_icons_installed(context);
        let snapshot = self.snapshot.clone();
        apply_theme(context, &snapshot.theme_mode);
        motion::apply_preference(context, snapshot.global_settings.reduce_motion);
        let tokens = tokens(context, &snapshot.theme_mode);
        let commands = &self.command_sender;
        let i18n = &self.i18n;

        if snapshot.migration_recovery.blocks_normal_shell() {
            TopBottomPanel::top("correo-recovery-command")
                .exact_height(layout::HEADER_HEIGHT)
                .frame(top_frame(tokens))
                .show(context, |ui| {
                    migration_recovery::top_bar(ui, &snapshot.migration_recovery);
                });

            SidePanel::left("correo-recovery-context")
                .default_width(layout::RECOVERY_CONTEXT_DEFAULT_WIDTH)
                .width_range(
                    layout::RECOVERY_CONTEXT_MIN_WIDTH..=layout::RECOVERY_CONTEXT_MAX_WIDTH,
                )
                .resizable(true)
                .frame(sidebar_frame(tokens))
                .show(context, |ui| {
                    migration_recovery::context_panel(ui, &snapshot.migration_recovery, tokens);
                });

            CentralPanel::default()
                .frame(central_frame(tokens))
                .show(context, |ui| {
                    migration_recovery::show(ui, &snapshot.migration_recovery, tokens, commands);
                });
            return;
        }

        let header = TopBottomPanel::top("correo-command")
            .exact_height(layout::HEADER_HEIGHT)
            .frame(top_frame(tokens))
            .show(context, |ui| {
                command_bar::command_bar_title(ui, i18n);
            });
        let klingon_unlocked = self.klingon_language_visible(&snapshot);

        SidePanel::left("correo-rail")
            .exact_width(layout::RAIL_WIDTH)
            .resizable(false)
            .frame(rail_frame(tokens))
            .show(context, |ui| {
                nav::rail(ui, snapshot.active_workspace, tokens, commands, i18n);
            });

        let compact_connections_context =
            responsive::connections_context_is_compact(context, snapshot.active_workspace);

        if context_panel_visible(&snapshot) && !compact_connections_context {
            if snapshot.active_workspace == Workspace::Connections {
                let sidebar_width = connection_sidebar_width(context);
                let response = SidePanel::left("correo-context")
                    .exact_width(sidebar_width)
                    .resizable(false)
                    .frame(sidebar_frame(tokens))
                    .show(context, |ui| {
                        connection_launcher::panel(ui, &snapshot, tokens, commands, i18n);
                    });
                connection_sidebar_resize_handle(
                    context,
                    response.response.rect,
                    response.response.layer_id,
                    tokens,
                    i18n,
                );
            } else {
                SidePanel::left("correo-context")
                    .default_width(layout::SIDEBAR_DEFAULT_WIDTH)
                    .width_range(layout::SIDEBAR_MIN_WIDTH..=layout::SIDEBAR_MAX_WIDTH)
                    .resizable(true)
                    .frame(sidebar_frame(tokens))
                    .show(context, |ui| {
                        workspace::sidebar(
                            ui,
                            &snapshot,
                            snapshot.active_workspace,
                            tokens,
                            commands,
                            i18n,
                        );
                    });
            }
        }

        let mut about_logo_triggered = false;
        CentralPanel::default()
            .frame(central_frame(tokens))
            .show(context, |ui| {
                workspace::show(
                    ui,
                    &snapshot,
                    tokens,
                    commands,
                    i18n,
                    self.payload_highlighter.as_ref(),
                    klingon_unlocked,
                    &mut about_logo_triggered,
                );
            });
        if about_logo_triggered {
            self.klingon_easter_egg_clicks = self.klingon_easter_egg_clicks.saturating_add(1);
        }
        let klingon_unlocked = self.klingon_language_visible(&snapshot);
        if compact_connections_context {
            connection_flyout(context, &snapshot, tokens, commands, i18n);
        }
        command_bar::command_bar_controls(
            context,
            header.response.rect,
            &snapshot,
            commands,
            i18n,
            klingon_unlocked,
        );
        transfer_wizard::show(
            context,
            &snapshot,
            tokens,
            commands,
            i18n,
            &mut self.transfer_wizard,
        );
        toasts::show(context, &snapshot, tokens);
    }

    fn ensure_icons_installed(&mut self, context: &egui::Context) {
        if !self.icons_installed {
            icons::install(context);
            self.icons_installed = true;
        }
    }

    fn klingon_language_visible(&self, snapshot: &AppSnapshot) -> bool {
        self.klingon_easter_egg_clicks >= 2 || snapshot.global_settings.language == "tlh"
    }
}

fn context_panel_visible(snapshot: &AppSnapshot) -> bool {
    !matches!(
        snapshot.active_workspace,
        Workspace::Scripts
            | Workspace::Plugins
            | Workspace::Diagnostics
            | Workspace::Settings
            | Workspace::About
    )
}

fn connection_sidebar_width(context: &egui::Context) -> f32 {
    context
        .data_mut(|data| {
            data.get_persisted(connection_sidebar_width_id())
                .unwrap_or(layout::CONNECTION_FLYOUT_WIDTH)
        })
        .clamp(
            layout::CONNECTION_SIDEBAR_MIN_WIDTH,
            layout::CONNECTION_SIDEBAR_MAX_WIDTH,
        )
}

fn connection_sidebar_resize_handle(
    context: &egui::Context,
    rect: egui::Rect,
    layer_id: egui::LayerId,
    tokens: ThemeTokens,
    i18n: &I18n,
) {
    let x = rect.right();
    let handle_width = layout::WORKBENCH_DIVIDER_SIZE;
    let divider_top_inset = f32::from(layout::SIDEBAR_MARGIN_TOP);
    let divider_bottom_inset =
        f32::from(layout::SIDEBAR_MARGIN_BOTTOM) + f32::from(layout::CENTRAL_MARGIN);
    let divider_rect = egui::Rect::from_min_max(
        egui::pos2(x - handle_width * 0.5, rect.top() + divider_top_inset),
        egui::pos2(x + handle_width * 0.5, rect.bottom() - divider_bottom_inset),
    );
    if widgets::flyout_mode_button_above_global_edge(
        context,
        "connections-flyout-mode-button",
        divider_rect.center().x,
        context.screen_rect().top() + layout::HEADER_HEIGHT,
        &i18n.text("connection-use-flyout"),
    )
    .clicked()
    {
        responsive::set_forced_context_flyout_mode(context, true);
        responsive::close_connection_flyout(context);
        motion::finish_flyout_closed(context, "connections-context");
    }
    egui::Area::new(egui::Id::new("connections-context-resize-handle"))
        .order(egui::Order::Middle)
        .fixed_pos(divider_rect.min)
        .movable(false)
        .show(context, |ui| {
            let handle_rect = egui::Rect::from_min_size(
                ui.min_rect().min,
                egui::vec2(handle_width, divider_rect.height()),
            );
            let resize_rect =
                egui::Rect::from_min_max(handle_rect.left_top(), handle_rect.right_bottom());
            let response = ui
                .allocate_rect(resize_rect, egui::Sense::click_and_drag())
                .on_hover_cursor(egui::CursorIcon::ResizeHorizontal);
            if response.dragged() {
                if let Some(pointer) = response.interact_pointer_pos() {
                    let width = (pointer.x - rect.left()).clamp(
                        layout::CONNECTION_SIDEBAR_MIN_WIDTH,
                        layout::CONNECTION_SIDEBAR_MAX_WIDTH,
                    );
                    ui.ctx().data_mut(|data| {
                        data.insert_persisted(connection_sidebar_width_id(), width)
                    });
                }
            }
        });

    context.layer_painter(layer_id).line_segment(
        [
            egui::pos2(x, rect.top() + divider_top_inset),
            egui::pos2(x, rect.bottom() - divider_bottom_inset),
        ],
        egui::Stroke::new(1.0, tokens.border),
    );
}

fn connection_sidebar_width_id() -> egui::Id {
    egui::Id::new("connections-context-sidebar-width")
}

fn connection_flyout(
    context: &egui::Context,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    let screen = context.screen_rect();
    let overlay_rect = egui::Rect::from_min_max(
        egui::pos2(
            screen.left() + layout::RAIL_WIDTH,
            screen.top() + layout::HEADER_HEIGHT,
        ),
        screen.right_bottom(),
    );
    if overlay_rect.width() <= 0.0 || overlay_rect.height() <= 0.0 {
        return;
    }

    let open = responsive::connection_flyout_open(context);
    if !open {
        connection_flyout_collapsed_handle(context, overlay_rect, i18n);
    }
    let Some(progress) = motion::flyout_progress(context, "connections-context", open) else {
        return;
    };
    if open && context.input(|input| input.key_pressed(egui::Key::Escape)) {
        responsive::close_connection_flyout(context);
    }

    egui::Area::new(egui::Id::new("connections-context-flyout"))
        .order(egui::Order::Foreground)
        .fixed_pos(overlay_rect.min)
        .movable(false)
        .show(context, |ui| {
            let (scrim_rect, _) = ui.allocate_exact_size(overlay_rect.size(), egui::Sense::hover());
            ui.painter().rect_filled(
                scrim_rect,
                egui::CornerRadius::ZERO,
                motion::scrim_color(crate::modal_style::SCRIM_ALPHA, progress),
            );

            let handle_rect = egui::Rect::from_min_size(
                scrim_rect.left_top(),
                egui::vec2(widgets::FLYOUT_HANDLE_WIDTH, scrim_rect.height()),
            );
            let panel_area = egui::Rect::from_min_max(
                egui::pos2(handle_rect.right(), scrim_rect.top()),
                scrim_rect.right_bottom(),
            );
            let panel_width = layout::CONNECTION_FLYOUT_WIDTH.min(panel_area.width());
            let panel_rect = motion::flyout_panel_rect(panel_area, panel_width, progress);
            ui.painter()
                .rect_filled(panel_rect, egui::CornerRadius::ZERO, tokens.window_bg);

            let margin = layout::sidebar_margin();
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
            connection_launcher::panel(&mut panel_ui, snapshot, tokens, commands, i18n);
            connection_flyout_expanded_controls(ui, handle_rect, panel_rect, i18n);

            let clicked_outside = open
                && ui.ctx().input(|input| {
                    input.pointer.any_click()
                        && input
                            .pointer
                            .interact_pos()
                            .is_some_and(|pos| !panel_rect.contains(pos))
                });
            if clicked_outside {
                responsive::close_connection_flyout(ui.ctx());
            }
        });
}

fn connection_flyout_collapsed_handle(
    context: &egui::Context,
    overlay_rect: egui::Rect,
    i18n: &I18n,
) {
    egui::Area::new(egui::Id::new("connections-context-flyout-collapsed-handle"))
        .order(egui::Order::Foreground)
        .fixed_pos(overlay_rect.min)
        .movable(false)
        .show(context, |ui| {
            let handle_rect = egui::Rect::from_min_size(
                ui.min_rect().min,
                egui::vec2(widgets::FLYOUT_HANDLE_WIDTH, overlay_rect.height()),
            );
            if widgets::flyout_handle(
                ui,
                handle_rect,
                "connections-flyout-open-handle",
                regular::CARET_RIGHT,
                &i18n.text("connection-show-list"),
            )
            .clicked()
            {
                responsive::open_connection_flyout(ui.ctx());
            }
        });
}

fn connection_flyout_expanded_controls(
    ui: &mut egui::Ui,
    handle_rect: egui::Rect,
    panel_rect: egui::Rect,
    i18n: &I18n,
) {
    if widgets::flyout_handle(
        ui,
        handle_rect,
        "connections-flyout-collapse-handle",
        regular::CARET_LEFT,
        &i18n.text("connection-collapse-flyout"),
    )
    .clicked()
    {
        responsive::close_connection_flyout(ui.ctx());
    }

    if !(responsive::forced_connection_flyout_mode(ui.ctx())
        && !responsive::connections_context_requires_flyout(ui.ctx()))
    {
        return;
    }

    if widgets::flyout_restore_button_above_edge(
        ui,
        "connections-flyout-restore-button",
        panel_rect.right(),
        panel_rect.top(),
        &i18n.text("connection-use-sidebar"),
    )
    .clicked()
    {
        responsive::set_forced_context_flyout_mode(ui.ctx(), false);
        responsive::close_connection_flyout(ui.ctx());
    }
}

impl Default for CorreoUi {
    fn default() -> Self {
        Self::for_theme(ThemeMode::System)
    }
}

impl eframe::App for CorreoUi {
    fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        self.draw(context);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, THEME_KEY, &self.snapshot.theme_mode);
    }
}

pub fn stored_theme(creation_context: &eframe::CreationContext<'_>) -> ThemeMode {
    creation_context
        .storage
        .and_then(|storage| eframe::get_value::<ThemeMode>(storage, THEME_KEY))
        .unwrap_or_default()
}

fn top_frame(tokens: ThemeTokens) -> Frame {
    Frame::NONE
        .fill(chrome_bg(tokens))
        .inner_margin(egui::Margin::symmetric(
            layout::HEADER_MARGIN_X,
            layout::HEADER_MARGIN_Y,
        ))
}

fn rail_frame(tokens: ThemeTokens) -> Frame {
    Frame::NONE
        .fill(chrome_bg(tokens))
        .inner_margin(egui::Margin::same(layout::RAIL_MARGIN))
}

fn chrome_bg(tokens: ThemeTokens) -> egui::Color32 {
    tokens.panel_raised.gamma_multiply(1.12)
}

fn sidebar_frame(tokens: ThemeTokens) -> Frame {
    Frame::NONE
        .fill(tokens.window_bg)
        .inner_margin(layout::sidebar_margin())
}

fn central_frame(tokens: ThemeTokens) -> Frame {
    Frame::NONE
        .fill(tokens.window_bg)
        .inner_margin(egui::Margin::same(layout::CENTRAL_MARGIN))
}

#[cfg(test)]
mod tests {
    use super::*;
    use correo_style::static_tokens;

    #[test]
    fn header_frame_uses_larger_height_and_horizontal_padding() {
        let frame = top_frame(static_tokens(&ThemeMode::Dark));

        assert_eq!(layout::HEADER_HEIGHT, 64.0);
        assert_eq!(
            frame.inner_margin,
            egui::Margin::symmetric(layout::HEADER_MARGIN_X, layout::HEADER_MARGIN_Y)
        );
    }

    #[test]
    fn global_chrome_frames_do_not_draw_decorative_strokes() {
        let tokens = static_tokens(&ThemeMode::Dark);

        assert_eq!(top_frame(tokens).stroke, egui::Stroke::NONE);
        assert_eq!(rail_frame(tokens).stroke, egui::Stroke::NONE);
        assert_eq!(sidebar_frame(tokens).stroke, egui::Stroke::NONE);
    }
}
