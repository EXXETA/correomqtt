use correo_core::{sample_snapshot, AppCommandSender, AppSnapshot, ThemeMode, Workspace};
use correo_style::{apply_theme, layout, tokens, ThemeTokens};
use egui::{Button, CentralPanel, Frame, RichText, SidePanel, TopBottomPanel};
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

        TopBottomPanel::top("correo-command")
            .exact_height(layout::HEADER_HEIGHT)
            .frame(top_frame(tokens))
            .show(context, |ui| {
                command_bar::command_bar(ui, &snapshot, tokens, commands, i18n);
            });

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
                connection_sidebar_resize_handle(context, response.response.rect, tokens);
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
                );
            });
        if compact_connections_context {
            connection_flyout(context, &snapshot, tokens, commands, i18n);
        }
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
    tokens: ThemeTokens,
) {
    let x = rect.right();
    let handle_width = layout::WORKBENCH_DIVIDER_SIZE;
    egui::Area::new(egui::Id::new("connections-context-resize-handle"))
        .order(egui::Order::Foreground)
        .fixed_pos(egui::pos2(x - handle_width * 0.5, rect.top()))
        .movable(false)
        .show(context, |ui| {
            let handle_rect = egui::Rect::from_min_size(
                ui.min_rect().min,
                egui::vec2(handle_width, rect.height()),
            );
            let response = ui
                .allocate_rect(handle_rect, egui::Sense::click_and_drag())
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

    context
        .layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::new("connections-context-resize-line"),
        ))
        .line_segment(
            [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
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
    let open = responsive::connection_flyout_open(context);
    let Some(progress) = motion::flyout_progress(context, "connections-context", open) else {
        return;
    };
    if open && context.input(|input| input.key_pressed(egui::Key::Escape)) {
        responsive::close_connection_flyout(context);
    }

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

            let panel_width = layout::CONNECTION_FLYOUT_WIDTH.min(overlay_rect.width());
            let panel_rect = motion::flyout_panel_rect(scrim_rect, panel_width, progress);
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
            connection_flyout_restore_button(ui, panel_rect);

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

fn connection_flyout_restore_button(ui: &mut egui::Ui, panel_rect: egui::Rect) {
    if !(responsive::forced_connection_flyout_mode(ui.ctx())
        && !responsive::connections_context_requires_flyout(ui.ctx()))
    {
        return;
    }

    let button_rect = egui::Rect::from_min_size(
        egui::pos2(
            panel_rect.right() + layout::TOOLBAR_GAP,
            panel_rect.top() + f32::from(layout::SIDEBAR_MARGIN_TOP),
        ),
        egui::Vec2::from(layout::square_icon_button_size()),
    );
    let mut button_ui = ui.new_child(egui::UiBuilder::new().max_rect(button_rect).layout(
        egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
    ));
    if button_ui
        .scope(|ui| {
            widgets::with_icon_button_padding(ui, |ui| {
                ui.add_sized(
                    layout::square_icon_button_size(),
                    Button::new(RichText::new(regular::SIDEBAR_SIMPLE).size(15.0)),
                )
            })
        })
        .inner
        .on_hover_text("Use connections sidebar")
        .clicked()
    {
        responsive::set_forced_connection_flyout_mode(ui.ctx(), false);
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
