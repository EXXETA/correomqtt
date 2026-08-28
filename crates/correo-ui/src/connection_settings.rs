use correo_core::{
    AppCommand, AppCommandSender, AppSnapshot, ConnectionSecretField, ConnectionSettingField,
    ConnectionSettingFlag, ConnectionSettingsSnapshot, ConnectionSettingsTab,
};
use correo_style::layout;
use egui::{
    Button, Color32, CornerRadius, Layout, Rect, RichText, ScrollArea, Sense, TextEdit, Ui,
    UiBuilder,
};
use egui_phosphor::regular;

use crate::widgets::{
    dotted_focus_outline, paint_top_tab_strip_underline, square_icon_button_size,
    with_icon_button_padding, TopUnderlineTab,
};
use crate::workbench_helpers::qos_selector;
use crate::{i18n::I18n, theme::ThemeTokens};

#[path = "connection_settings_actions.rs"]
mod actions;
#[path = "connection_settings_controls.rs"]
mod controls;
use actions::{action_bar, delete_confirmation};
use controls::{
    combo, control_width, field, field_with_button, file_field, flag, row, secret_field,
    secret_field_enabled, send,
};

const MODAL_MAX_WIDTH: f32 = controls::FORM_MAX_WIDTH + 24.0;
const MODAL_HEIGHT_SCALE: f32 = 0.9;
const MODAL_MAX_HEIGHT: f32 = 720.0;
const SCRIM_ALPHA: u8 = 176;
const MODAL_RADIUS: u8 = 4;
const MODAL_PADDING: i8 = 12;
const TAB_GAP: f32 = 4.0;

pub fn show(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    show_body(ui, snapshot, tokens, commands, i18n, false);
}

fn show_body(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
    modal: bool,
) {
    let settings = &snapshot.connection_settings;
    header(ui, i18n);
    ui.add_space(8.0);
    settings_content(ui, settings, tokens, commands, i18n);
    ui.add_space(8.0);
    action_bar(
        ui,
        settings,
        commands,
        i18n,
        modal,
        !is_broker_settings(settings),
    );

    if settings.delete_confirmation_open {
        delete_confirmation(ui, settings, tokens, commands, i18n);
    }
}

pub fn overlay(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    if snapshot.connection_settings.delete_confirmation_open
        && snapshot.connection_settings_overlay.is_none()
        && snapshot.selected_connection.is_some()
    {
        delete_confirmation(ui, &snapshot.connection_settings, tokens, commands, i18n);
        return;
    }

    let overlay_key = if let Some(editor_id) = snapshot.connection_settings_overlay {
        if snapshot.selected_connection != Some(editor_id) {
            return;
        }
        editor_id.to_string()
    } else if snapshot.connection_surface == correo_core::ConnectionSurface::Settings
        && snapshot.selected_connection.is_none()
    {
        "new-connection".to_owned()
    } else {
        return;
    };
    if ui.ctx().input(|input| input.key_pressed(egui::Key::Escape)) {
        if snapshot.connection_settings.delete_confirmation_open {
            send(commands, AppCommand::CancelDeleteConnection);
        } else {
            send(commands, AppCommand::DiscardConnectionSettings);
        }
    }

    let overlay_rect = crate::overlay_bounds::get(ui);
    let modal_size = egui::vec2(
        (overlay_rect.width() * 0.95).min(MODAL_MAX_WIDTH),
        (overlay_rect.height() * MODAL_HEIGHT_SCALE).min(MODAL_MAX_HEIGHT),
    );
    let is_new_draft = snapshot.selected_connection.is_none();
    let body_id = egui::Id::new(("connection-settings-overlay-body", overlay_key.clone()));
    egui::Area::new(egui::Id::new(("connection-settings-overlay", overlay_key)))
        .order(egui::Order::Foreground)
        .fixed_pos(overlay_rect.min)
        .movable(false)
        .show(ui.ctx(), |ui| {
            let (scrim_rect, _) = ui.allocate_exact_size(overlay_rect.size(), Sense::click());
            ui.painter().rect_filled(
                scrim_rect,
                CornerRadius::ZERO,
                Color32::from_black_alpha(SCRIM_ALPHA),
            );

            let modal_rect = Rect::from_center_size(scrim_rect.center(), modal_size);
            ui.painter().rect_filled(
                modal_rect,
                CornerRadius::same(MODAL_RADIUS),
                modal_bg(tokens),
            );

            let content_rect = modal_rect.shrink(f32::from(MODAL_PADDING));
            let mut content_ui = ui.new_child(UiBuilder::new().max_rect(content_rect));
            content_ui.set_min_size(content_rect.size());
            content_ui.set_max_size(content_rect.size());
            content_ui.set_clip_rect(content_rect);
            content_ui.vertical(|ui| {
                modal_header(ui, commands, i18n);
                ui.separator();
                let footer_height = crate::theme::CONTROL_HEIGHT + 20.0;
                let body_height = (ui.available_height() - footer_height).max(120.0);
                ScrollArea::vertical()
                    .id_salt(body_id)
                    .max_height(body_height)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        settings_content(ui, &snapshot.connection_settings, tokens, commands, i18n);
                    });
                ui.add_space(8.0);
                action_bar(
                    ui,
                    &snapshot.connection_settings,
                    commands,
                    i18n,
                    true,
                    !is_new_draft && !is_broker_settings(&snapshot.connection_settings),
                );
                if snapshot.connection_settings.delete_confirmation_open {
                    delete_confirmation(ui, &snapshot.connection_settings, tokens, commands, i18n);
                }
            });
        });
}

fn header(ui: &mut Ui, i18n: &I18n) {
    ui.heading(i18n.text("connection-settings-title"));
}

fn modal_bg(tokens: ThemeTokens) -> Color32 {
    let color = tokens.window_bg;
    Color32::from_rgb(color.r(), color.g(), color.b())
}

fn modal_header(ui: &mut Ui, commands: &AppCommandSender, i18n: &I18n) {
    ui.horizontal(|ui| {
        ui.heading(i18n.text("connection-settings-title"));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if with_icon_button_padding(ui, |ui| {
                ui.add_sized(
                    square_icon_button_size(),
                    Button::new(RichText::new(regular::X).size(16.0)),
                )
            })
            .on_hover_text(i18n.text("common-cancel"))
            .clicked()
            {
                send(commands, AppCommand::DiscardConnectionSettings);
            }
        });
    });
}

fn settings_content(
    ui: &mut Ui,
    settings: &ConnectionSettingsSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    tab_bar(ui, settings.selected_tab, tokens, commands, i18n);
    ui.add_space(8.0);
    match settings.selected_tab {
        ConnectionSettingsTab::Mqtt => mqtt_tab(ui, settings, tokens, commands, i18n),
        ConnectionSettingsTab::Tls => tls_tab(ui, settings, tokens, commands, i18n),
        ConnectionSettingsTab::Proxy => proxy_tab(ui, settings, tokens, commands, i18n),
        ConnectionSettingsTab::Lwt => lwt_tab(ui, settings, tokens, commands, i18n),
    }
    ui.add_space(8.0);
}

fn tab_bar(
    ui: &mut Ui,
    selected: ConnectionSettingsTab,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    let tab_count = ConnectionSettingsTab::ALL.len() as f32;
    let strip_width = ui.available_width().clamp(0.0, controls::FORM_MAX_WIDTH);
    let strip_rect = Rect::from_min_size(
        ui.available_rect_before_wrap().min,
        egui::vec2(strip_width, layout::CONTROL_HEIGHT),
    );
    ui.allocate_rect(strip_rect, Sense::hover());

    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(strip_rect)
            .layout(Layout::left_to_right(egui::Align::Center)),
    );
    child.set_clip_rect(strip_rect);
    child.spacing_mut().item_spacing.x = 0.0;

    let tab_width = ((strip_width - (TAB_GAP * (tab_count - 1.0))) / tab_count).max(0.0);
    let mut active_rect = None;
    let mut focused_rect = None;
    for (index, tab) in ConnectionSettingsTab::ALL.into_iter().enumerate() {
        let is_selected = selected == tab;
        let label = i18n.connection_settings_tab_label(tab);
        let label = if is_selected {
            RichText::new(label).strong()
        } else {
            RichText::new(label)
        };
        let response = child.add(
            TopUnderlineTab::new(label, is_selected, tokens)
                .size(egui::vec2(tab_width, layout::CONTROL_HEIGHT)),
        );
        if is_selected {
            active_rect = Some(response.rect);
        }
        if response.has_focus() {
            focused_rect = Some(response.rect);
        }
        if response.clicked() {
            send(commands, AppCommand::SelectConnectionSettingsTab(tab));
        }
        if index + 1 < ConnectionSettingsTab::ALL.len() {
            child.add_space(TAB_GAP);
        }
    }
    if let Some(active_rect) = active_rect {
        paint_top_tab_strip_underline(&child, strip_rect, active_rect, tokens);
    }
    if let Some(focused_rect) = focused_rect {
        dotted_focus_outline(&child, focused_rect);
    }
}

fn mqtt_tab(
    ui: &mut Ui,
    settings: &ConnectionSettingsSnapshot,
    _tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    field(
        ui,
        &required_label(i18n.text("connection-name")),
        &settings.profile_name,
        ConnectionSettingField::ProfileName,
        commands,
    );
    field(
        ui,
        &required_label(i18n.text("connection-host")),
        &settings.host,
        ConnectionSettingField::Host,
        commands,
    );
    field(
        ui,
        &required_label(i18n.text("connection-port")),
        &settings.port,
        ConnectionSettingField::Port,
        commands,
    );
    combo(
        ui,
        &i18n.text("connection-mqtt-version"),
        &settings.mqtt_version,
        ConnectionSettingField::MqttVersion,
        &["MQTT 3.1.1", "MQTT v5"],
        commands,
    );
    flag(
        ui,
        &i18n.text("connection-clean-session"),
        settings.clean_session,
        ConnectionSettingFlag::CleanSession,
        commands,
    );
    field_with_button(
        ui,
        &i18n.text("connection-client-id"),
        &settings.client_id,
        ConnectionSettingField::ClientId,
        &i18n.text("connection-generate"),
        AppCommand::GenerateClientId,
        commands,
    );
    field(
        ui,
        &i18n.text("connection-username"),
        &settings.username,
        ConnectionSettingField::Username,
        commands,
    );
    secret_field(
        ui,
        &i18n.text("connection-password"),
        &settings.password,
        ConnectionSecretField::MqttPassword,
        commands,
    );
}

fn tls_tab(
    ui: &mut Ui,
    settings: &ConnectionSettingsSnapshot,
    _tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    combo(
        ui,
        &i18n.text("connection-tls-ssl"),
        &settings.tls_mode,
        ConnectionSettingField::TlsMode,
        &["No TLS/SSL", "Keystore"],
        commands,
    );
    file_field(
        ui,
        &maybe_required_label(
            i18n.text("connection-ssl-keystore"),
            settings.tls_mode == "Keystore",
        ),
        &settings.tls_store,
        ConnectionSettingField::TlsStore,
        true,
        commands,
    );
    secret_field(
        ui,
        &i18n.text("connection-ssl-password"),
        &settings.tls_keystore_password,
        ConnectionSecretField::TlsKeystorePassword,
        commands,
    );
    flag(
        ui,
        &i18n.text("connection-verify-hostname"),
        settings.tls_host_verification,
        ConnectionSettingFlag::TlsHostVerification,
        commands,
    );
}

fn proxy_tab(
    ui: &mut Ui,
    settings: &ConnectionSettingsSnapshot,
    _tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    combo(
        ui,
        &i18n.text("connection-proxy-mode"),
        &settings.proxy_mode,
        ConnectionSettingField::ProxyMode,
        &["No proxy/tunnel", "SSH"],
        commands,
    );
    ui.add_enabled_ui(settings.proxy_mode == "SSH", |ui| {
        field(
            ui,
            &maybe_required_label(
                i18n.text("connection-ssh-host"),
                settings.proxy_mode == "SSH",
            ),
            &settings.ssh_host,
            ConnectionSettingField::SshHost,
            commands,
        );
        field(
            ui,
            &maybe_required_label(
                i18n.text("connection-ssh-port"),
                settings.proxy_mode == "SSH",
            ),
            &settings.ssh_port,
            ConnectionSettingField::SshPort,
            commands,
        );
        field(
            ui,
            &i18n.text("connection-local-mqtt-port"),
            &settings.local_mqtt_port,
            ConnectionSettingField::LocalMqttPort,
            commands,
        );
        combo(
            ui,
            &i18n.text("connection-authentication"),
            &settings.auth_mode,
            ConnectionSettingField::AuthMode,
            &["No Auth", "Keyfile", "Password"],
            commands,
        );
        field(
            ui,
            &maybe_required_label(
                i18n.text("connection-ssh-username"),
                settings.auth_mode != "No Auth",
            ),
            &settings.auth_username,
            ConnectionSettingField::AuthUsername,
            commands,
        );
        secret_field_enabled(
            ui,
            &i18n.text("connection-ssh-password"),
            &settings.ssh_password,
            ConnectionSecretField::SshPassword,
            settings.auth_mode != "No Auth",
            commands,
        );
        file_field(
            ui,
            &maybe_required_label(
                i18n.text("connection-ssh-key-file"),
                settings.auth_mode == "Keyfile",
            ),
            &settings.ssh_key_file,
            ConnectionSettingField::SshKeyFile,
            settings.auth_mode == "Keyfile",
            commands,
        );
    });
}

fn lwt_tab(
    ui: &mut Ui,
    settings: &ConnectionSettingsSnapshot,
    _tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    let mut enabled = settings.lwt_enabled;
    row(ui, &i18n.text("connection-enable-last-will"), |ui| {
        if crate::widgets::checkbox(ui, &mut enabled, "").changed() {
            send(commands, AppCommand::SetLwtEnabled(enabled));
        }
    });
    ui.add_enabled_ui(settings.lwt_enabled, |ui| {
        field(
            ui,
            &i18n.text("connection-lwt-topic"),
            &settings.lwt_topic,
            ConnectionSettingField::LwtTopic,
            commands,
        );
        row(ui, "QoS", |ui| {
            qos_selector(ui, "connection-lwt-qos", settings.lwt_qos, |qos| {
                send(commands, AppCommand::UpdateLwtQos(qos));
            });
        });
        flag(
            ui,
            &i18n.text("connection-lwt-retained"),
            settings.lwt_retained,
            ConnectionSettingFlag::LwtRetained,
            commands,
        );
        row(ui, &i18n.text("connection-lwt-payload"), |ui| {
            let mut payload = settings.lwt_payload.clone();
            if ui
                .add_sized(
                    [control_width(ui), 120.0],
                    crate::widgets::padded_text_edit(TextEdit::multiline(&mut payload))
                        .font(egui::TextStyle::Monospace)
                        .desired_rows(5)
                        .desired_width(f32::INFINITY),
                )
                .changed()
            {
                send(
                    commands,
                    AppCommand::UpdateConnectionSetting {
                        field: ConnectionSettingField::LwtPayload,
                        value: payload,
                    },
                );
            }
        });
    });
}

fn required_label(label: String) -> String {
    maybe_required_label(label, true)
}

fn is_broker_settings(settings: &ConnectionSettingsSnapshot) -> bool {
    settings.profile_name == correo_core::BUILT_IN_BROKER_CONNECTION_NAME
}

fn maybe_required_label(label: String, required: bool) -> String {
    if required {
        format!("{label} *")
    } else {
        label
    }
}
