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
