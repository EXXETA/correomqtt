use correo_core::{sample_snapshot, AppCommandSender, AppSnapshot, ThemeMode};
use correo_style::{apply_theme, layout, tokens, ThemeTokens};
use egui::{CentralPanel, Frame, SidePanel, TopBottomPanel};

use crate::{
    i18n::I18n, icons, migration_recovery, motion, nav, responsive, toasts, transfer_wizard,
    widgets::paint_pane_divider, workspace, PayloadHighlighter,
};

pub const THEME_KEY: &str = "correo.theme-mode";
const SIDEBAR_DIVIDER_HIT_WIDTH: f32 = 10.0;

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
                .frame(Frame::NONE.fill(chrome_bg(tokens)))
                .show(context, |ui| {
                    migration_recovery::top_bar(ui, &snapshot.migration_recovery);
                });

            let recovery_width = stored_panel_width(
                context,
                "recovery-context-width",
                layout::RECOVERY_CONTEXT_DEFAULT_WIDTH,
                layout::RECOVERY_CONTEXT_MIN_WIDTH,
                layout::RECOVERY_CONTEXT_MAX_WIDTH,
            );
            let recovery_panel = SidePanel::left("correo-recovery-context")
                .exact_width(recovery_width)
                .resizable(false)
                .show_separator_line(false)
                .frame(sidebar_frame(tokens))
                .show(context, |ui| {
                    migration_recovery::context_panel(ui, &snapshot.migration_recovery, tokens);
                });
            resizable_panel_divider(
                context,
                recovery_panel.response.rect,
                "recovery-context-divider",
                "recovery-context-width",
                layout::RECOVERY_CONTEXT_MIN_WIDTH,
                layout::RECOVERY_CONTEXT_MAX_WIDTH,
                tokens,
            );

            CentralPanel::default()
                .frame(central_frame(tokens))
                .show(context, |ui| {
                    migration_recovery::show(ui, &snapshot.migration_recovery, tokens, commands);
                });
            return;
        }

        macos_titlebar_spacer(context, tokens);
        let klingon_unlocked = self.klingon_language_visible(&snapshot);

        SidePanel::left("correo-rail")
            .exact_width(layout::RAIL_WIDTH)
            .resizable(false)
            .frame(rail_frame(tokens))
            .show(context, |ui| {
                nav::rail(ui, &snapshot, tokens, commands, i18n);
            });

        if responsive::toggle_sidebar_shortcut(context, snapshot.active_workspace) {
            responsive::toggle_sidebar(context);
        }
        let sidebar_visible = responsive::workspace_has_sidebar(snapshot.active_workspace)
            && responsive::sidebar_visible(context);
        let sidebar_uses_exclusive_content = responsive::sidebar_uses_exclusive_content(context);
        let mut about_logo_triggered = false;

        if sidebar_uses_exclusive_content && sidebar_visible {
            CentralPanel::default()
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
        } else {
            let sidebar_width = stored_panel_width(
                context,
                "context-sidebar-width",
                layout::SIDEBAR_DEFAULT_WIDTH,
                layout::SIDEBAR_MIN_WIDTH,
                layout::SIDEBAR_MAX_WIDTH,
            );
            let sidebar_panel = SidePanel::left("correo-context")
                .exact_width(sidebar_width)
                .resizable(false)
                .show_separator_line(false)
                .frame(sidebar_frame(tokens))
                .show_animated(context, sidebar_visible, |ui| {
                    workspace::sidebar(
                        ui,
                        &snapshot,
                        snapshot.active_workspace,
                        tokens,
                        commands,
                        i18n,
                    );
                });
            if let Some(sidebar_panel) = sidebar_panel {
                resizable_panel_divider(
                    context,
                    sidebar_panel.response.rect,
                    "context-sidebar-divider",
                    "context-sidebar-width",
                    layout::SIDEBAR_MIN_WIDTH,
                    layout::SIDEBAR_MAX_WIDTH,
                    tokens,
                );
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
                        klingon_unlocked,
                        &mut about_logo_triggered,
                    );
                });
        }
        if about_logo_triggered {
            self.klingon_easter_egg_clicks = self.klingon_easter_egg_clicks.saturating_add(1);
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

    fn klingon_language_visible(&self, snapshot: &AppSnapshot) -> bool {
        self.klingon_easter_egg_clicks >= 2 || snapshot.global_settings.language == "tlh"
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

fn macos_titlebar_spacer(context: &egui::Context, tokens: ThemeTokens) {
    #[cfg(target_os = "macos")]
    TopBottomPanel::top("correo-macos-titlebar")
        .exact_height(28.0)
        .frame(Frame::NONE.fill(chrome_bg(tokens)))
        .show(context, |_| {});

    #[cfg(not(target_os = "macos"))]
    let _ = (context, tokens);
}

fn stored_panel_width(
    context: &egui::Context,
    id: &'static str,
    default: f32,
    min: f32,
    max: f32,
) -> f32 {
    context
        .data_mut(|data| *data.get_persisted_mut_or(egui::Id::new(id), default))
        .clamp(min, max)
}

fn resizable_panel_divider(
    context: &egui::Context,
    panel_rect: egui::Rect,
    divider_id: &'static str,
    width_id: &'static str,
    min_width: f32,
    max_width: f32,
    tokens: ThemeTokens,
) {
    let hit_rect = egui::Rect::from_center_size(
        egui::pos2(panel_rect.right(), panel_rect.center().y),
        egui::vec2(SIDEBAR_DIVIDER_HIT_WIDTH, panel_rect.height()),
    );
    egui::Area::new(egui::Id::new(divider_id))
        .order(egui::Order::Foreground)
        .fixed_pos(hit_rect.min)
        .movable(false)
        .show(context, |ui| {
            let (rect, response) =
                ui.allocate_exact_size(hit_rect.size(), egui::Sense::click_and_drag());
            let response = response.on_hover_cursor(egui::CursorIcon::ResizeHorizontal);
            if response.dragged() {
                if let Some(pointer) = response.interact_pointer_pos() {
                    let width = (pointer.x - panel_rect.left()).clamp(min_width, max_width);
                    context.data_mut(|data| data.insert_persisted(egui::Id::new(width_id), width));
                }
            }
            paint_pane_divider(ui, rect, &response, tokens, true);
        });
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
        .fill(sidebar_bg(tokens))
        .inner_margin(layout::sidebar_margin())
}

fn sidebar_bg(tokens: ThemeTokens) -> egui::Color32 {
    let [red, green, blue, _] = tokens.window_bg.to_array();
    let luminance = (u16::from(red) + u16::from(green) + u16::from(blue)) / 3;
    if luminance < 128 {
        tokens.window_bg.gamma_multiply(1.5)
    } else {
        tokens.window_bg.gamma_multiply(0.88)
    }
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
    fn sidebar_frame_contrasts_with_the_main_content_in_both_themes() {
        let dark = static_tokens(&ThemeMode::Dark);
        let light = static_tokens(&ThemeMode::Light);

        assert!(sidebar_bg(dark).r() >= dark.window_bg.r() + 8);
        assert!(sidebar_bg(light).r() + 16 <= light.window_bg.r());
    }

    #[test]
    fn global_chrome_frames_do_not_draw_decorative_strokes() {
        let tokens = static_tokens(&ThemeMode::Dark);

        assert_eq!(rail_frame(tokens).stroke, egui::Stroke::NONE);
        assert_eq!(sidebar_frame(tokens).stroke, egui::Stroke::NONE);
    }
}
