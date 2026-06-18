use correo_mqtt::ConnectionId;

use crate::{
    AppSnapshot, ConnectDisabledReason, ConnectionBadge, ConnectionSettingsSnapshot,
    ConnectionState, ConnectionSummary, Diagnostic, GlobalSettingsSnapshot, ImportPasswordState,
    KeyringState, MessageInspectorTab, MessageRow, MessageTransferSnapshot, PluginRepositoryRow,
    PublishHistoryRow, PublishPaneSnapshot, QosLevel, SubscribePaneSnapshot, SubscriptionRow,
    ThemeMode, TransferConnectionRow, TransferConnectionStatus, TransferFeedback,
    TransferFileSnapshot, TransferOutcome, TransferSection, TransferStep, TransferSurfaceSnapshot,
    WorkbenchSnapshot,
};

#[path = "samples/plugins.rs"]
mod plugins;
use plugins::sample_plugins;
#[path = "samples/scripts.rs"]
mod scripts;
use scripts::sample_scripts;

pub fn sample_snapshot(theme_mode: ThemeMode) -> AppSnapshot {
    let connections = vec![
        sample_connection("Local Broker", "local...:1883", ConnectionState::Connected),
        sample_connection("QA TLS", "qa-...:8883", ConnectionState::Disconnected),
        sample_connection("Staging MQTT5", "stage...:8883", ConnectionState::Error),
        sample_connection("Edge Lab", "edge...:1883", ConnectionState::Reconnecting),
    ];
    let selected_connection = connections.first().map(|connection| connection.id);
    let mut snapshot = AppSnapshot::empty();
    snapshot.active_connection = selected_connection;
    snapshot.connection_count = connections.len();
    snapshot.connections = connections;
    snapshot.diagnostics = vec![
        Diagnostic::warning("Old plugin state was reinitialized from bundled manifests."),
        Diagnostic::info("Local Broker accepted a synthetic connection snapshot."),
    ];
    snapshot.global_settings = sample_global_settings();
    snapshot.plugins = sample_plugins();
    snapshot.scripts = sample_scripts(selected_connection);
    snapshot.selected_connection = selected_connection;
    snapshot.theme_mode = theme_mode;
    snapshot.transfer = sample_transfer();
    snapshot.workbench = sample_workbench();
    snapshot.connection_settings = sample_connection_settings();
    snapshot
}

fn sample_connection(name: &str, endpoint: &str, state: ConnectionState) -> ConnectionSummary {
    let badges = match name {
        "QA TLS" => vec![ConnectionBadge::Credentials, ConnectionBadge::Tls],
        "Staging MQTT5" => vec![ConnectionBadge::Tls, ConnectionBadge::Lwt],
        "Edge Lab" => vec![ConnectionBadge::Proxy],
        _ => vec![],
    };
    let disabled_reason = match state {
        ConnectionState::Connected => Some(ConnectDisabledReason::AlreadyConnected),
        ConnectionState::Reconnecting => Some(ConnectDisabledReason::Busy),
        _ => None,
    };

    ConnectionSummary {
        id: ConnectionId::new(),
        name: name.to_owned(),
        endpoint: endpoint.to_owned(),
        mqtt_version: if name == "Staging MQTT5" {
            "MQTT v5"
        } else {
            "MQTT 3.1.1"
        }
        .to_owned(),
        badges,
        active_plugin_workflows: name == "Local Broker",
        state,
        disabled_reason,
        recent_subscriptions: if name == "Local Broker" { 5 } else { 2 },
        recent_messages: if name == "Local Broker" { 128 } else { 18 },
        last_activity: match name {
            "Local Broker" => "Connected, 2 min uptime",
            "Staging MQTT5" => "Last error: TLS handshake failed",
            "Edge Lab" => "Reconnect scheduled in 14 s",
            _ => "Ready",
        }
        .to_owned(),
    }
}

fn sample_workbench() -> WorkbenchSnapshot {
    WorkbenchSnapshot {
        publish: PublishPaneSnapshot {
            topic: "telemetry/device-42/set".to_owned(),
            topic_history: vec![
                "telemetry/device-42/set".to_owned(),
                "telemetry/device-42/reboot".to_owned(),
                "lab/+/command".to_owned(),
            ],
            valid: true,
            qos: QosLevel::One,
            retained: false,
            payload: "{\n  \"target\": \"pump\",\n  \"enabled\": true\n}".to_owned(),
            validation: vec!["Topic is valid".to_owned(), "Payload: 45 bytes".to_owned()],
            feedback: None,
            history_filter: String::new(),
            selected_history_id: Some(1),
            history: vec![
                history(
                    1,
                    "telemetry/device-42/set",
                    "10:24:12",
                    QosLevel::One,
                    false,
                    "{\n  \"target\": \"pump\",\n  \"enabled\": true\n}",
                    45,
                ),
                history(
                    2,
                    "lab/line-a/command",
                    "10:16:54",
                    QosLevel::Zero,
                    false,
                    "start line a",
                    18,
                ),
                history(
                    3,
                    "retain/config",
                    "09:58:01",
                    QosLevel::One,
                    true,
                    "retained configuration payload",
                    128,
                ),
            ],
        },
        subscribe: SubscribePaneSnapshot {
            topic: "telemetry/#".to_owned(),
            topic_history: vec![
                "telemetry/#".to_owned(),
                "alerts/+".to_owned(),
                "$SYS/broker/clients/#".to_owned(),
            ],
            valid: true,
            qos: QosLevel::One,
            validation: vec!["Topic filter is valid".to_owned()],
            feedback: None,
            subscriptions: vec![
                subscription("telemetry/#", QosLevel::One, 128),
                subscription("alerts/+", QosLevel::One, 8),
                subscription("$SYS/broker/clients/#", QosLevel::Zero, 34),
            ],
            unsubscribe_all_confirmation_count: None,
            message_filter: String::new(),
        },
        messages: vec![
            message(
                1,
                "telemetry/device-42/state",
                "10:25:02",
                QosLevel::One,
                false,
                "{\"online\":true,\"rpm\":1420}",
                27,
                &["validated", "json"],
            ),
            message(
                2,
                "alerts/line-a",
                "10:24:58",
                QosLevel::One,
                false,
                "temperature threshold exceeded",
                30,
                &["validator"],
            ),
            message(
                3,
                "$SYS/broker/clients/connected",
                "10:24:44",
                QosLevel::Zero,
                true,
                "18",
                2,
                &["retained"],
            ),
            message(
                4,
                "telemetry/device-77/state",
                "10:24:11",
                QosLevel::Two,
                false,
                "{\"online\":false,\"reason\":\"maintenance\"}",
                39,
                &["json", "formatted"],
            ),
        ],
        selected_message_id: Some(1),
        inspector_tab: MessageInspectorTab::Payload,
        detail: Default::default(),
        narrow_tab: crate::WorkbenchTab::Publish,
        reconnect_status: "Uptime 00:02:18".to_owned(),
    }
}

fn sample_connection_settings() -> ConnectionSettingsSnapshot {
    ConnectionSettingsSnapshot {
        internal_id: "local-broker-01".to_owned(),
        profile_name: "Local Broker".to_owned(),
        host: "local...".to_owned(),
        port: "1883".to_owned(),
        mqtt_version: "MQTT 3.1.1".to_owned(),
        clean_session: true,
        client_id: "correomqtt-desktop".to_owned(),
        username: "local-user".to_owned(),
        password_status: "MQTT password managed by keyring".to_owned(),
        tls_mode: "No TLS/SSL".to_owned(),
        tls_password_status: "No SSL password configured".to_owned(),
        tls_host_verification: true,
        proxy_mode: "No proxy/tunnel".to_owned(),
        ssh_port: "22".to_owned(),
        local_mqtt_port: "1883".to_owned(),
        auth_mode: "No Auth".to_owned(),
        ssh_password_status: "No SSH password configured".to_owned(),
        lwt_enabled: true,
        lwt_topic: "status/correomqtt".to_owned(),
        lwt_retained: false,
        lwt_payload: "{\"online\":false}".to_owned(),
        dirty: true,
        valid: false,
        save_disabled_reason: "Resolve validation errors before saving".to_owned(),
        keyring_state: KeyringState::Available,
        validation_errors: vec!["Client id cannot contain spaces in imported profiles".to_owned()],
        ..ConnectionSettingsSnapshot::default()
    }
}

fn sample_transfer() -> TransferSurfaceSnapshot {
    let import_rows = vec![
        transfer_row(
            "local-broker",
            "Local Broker",
            "local...:1883",
            "MQTT 3.1.1",
            true,
            TransferConnectionStatus::Update,
        ),
        transfer_row(
            "qa-tls",
            "QA TLS",
            "qa-...:8883",
            "MQTT 3.1.1",
            true,
            TransferConnectionStatus::MissingSecret,
        ),
        transfer_row(
            "edge-lab",
            "Edge Lab",
            "edge...:1883",
            "MQTT v5",
            false,
            TransferConnectionStatus::New,
        ),
    ];
    let export_rows = vec![
        transfer_row(
            "local-broker",
            "Local Broker",
            "local...:1883",
            "MQTT 3.1.1",
            true,
            TransferConnectionStatus::Exportable,
        ),
        transfer_row(
            "qa-tls",
            "QA TLS",
            "qa-...:8883",
            "MQTT 3.1.1",
            true,
            TransferConnectionStatus::Exportable,
        ),
        transfer_row(
            "staging-mqtt5",
            "Staging MQTT5",
            "stage...:8883",
            "MQTT v5",
            true,
            TransferConnectionStatus::Exportable,
        ),
    ];
    TransferSurfaceSnapshot {
        active_section: TransferSection::Import,
        active_step: TransferStep::Review,
        selected_connections: 3,
        encrypted_export: true,
        import: crate::ConnectionImportSnapshot {
            file: Some(TransferFileSnapshot {
                display_name: "sample-connections.cqc".to_owned(),
                path_hint: "Downloads/sample-connections.cqc".to_owned(),
                byte_size: 42_880,
                detected_connections: 3,
                encrypted: true,
            }),
            encrypted: true,
            password_state: ImportPasswordState::Accepted,
            rows: import_rows,
            warnings: vec![
                "QA TLS has auth metadata; secret values stay outside the UI snapshot.".to_owned(),
                "Edge Lab uses a proxy option that needs review before first connect.".to_owned(),
            ],
            feedback: Some(TransferFeedback::info(
                "Encrypted import unlocked; review profiles before applying.",
            )),
            outcome: None,
        },
        export: crate::ConnectionExportSnapshot {
            rows: export_rows,
            output_path: "Exports/correomqtt-connections.cqc".to_owned(),
            encrypted: true,
            password_confirmation: crate::ExportPasswordConfirmation::Confirmed,
            feedback: Some(TransferFeedback::info(
                "Encrypted export protects connection auth metadata in transit.",
            )),
            ..Default::default()
        },
        messages: MessageTransferSnapshot {
            import_file: Some(TransferFileSnapshot {
                display_name: "message-history.json".to_owned(),
                path_hint: "Downloads/message-history.json".to_owned(),
                byte_size: 18_432,
                detected_connections: 0,
                encrypted: false,
            }),
            export_path: "Exports/message-history.json".to_owned(),
            selected_messages: 24,
            available_messages: 128,
            feedback: Some(TransferFeedback::info(
                "Message archives include topics, QoS, retain flags, and payload bytes.",
            )),
            outcome: Some(TransferOutcome::success(
                "Message export preview ready",
                "24 retained message snapshots selected for export.",
            )),
        },
        warnings: vec![
            "Imported connection auth metadata excludes secret values".to_owned(),
            "Plain export excludes sensitive authentication values".to_owned(),
        ],
    }
}

fn transfer_row(
    id: &str,
    name: &str,
    endpoint: &str,
    mqtt_version: &str,
    selected: bool,
    status: TransferConnectionStatus,
) -> TransferConnectionRow {
    TransferConnectionRow {
        id: id.to_owned(),
        name: name.to_owned(),
        endpoint: endpoint.to_owned(),
        mqtt_version: mqtt_version.to_owned(),
        selected,
        status,
    }
}

fn sample_global_settings() -> GlobalSettingsSnapshot {
    GlobalSettingsSnapshot {
        language: "en_US".to_owned(),
        keyring_backend: "os".to_owned(),
        update_checks_enabled: true,
        last_update_check: "Last check failed: offline".to_owned(),
        cleanup_status: "Sensitive cleanup requires confirmation".to_owned(),
        search_use_regex: true,
        search_ignore_case: true,
        reduce_motion: false,
        bundled_plugins_url: "https://github.com/farion/correomqtt-rust/releases".to_owned(),
        plugin_repositories: vec![PluginRepositoryRow {
            id: "sample".to_owned(),
            url: "https://example.invalid/plugins.json".to_owned(),
        }],
        config_version: "0.0.0-sample".to_owned(),
        window_geometry: "1280x800 at 80,60".to_owned(),
        ..GlobalSettingsSnapshot::default()
    }
}

fn history(
    id: u32,
    topic: &str,
    timestamp: &str,
    qos: QosLevel,
    retained: bool,
    payload_preview: &str,
    byte_size: usize,
) -> PublishHistoryRow {
    let payload = payload_preview.as_bytes().to_vec();
    let mut badges = Vec::new();
    if retained {
        badges.push("retained".to_owned());
    }
    PublishHistoryRow {
        id,
        topic: topic.to_owned(),
        timestamp: timestamp.to_owned(),
        qos,
        retained,
        payload,
        payload_preview: payload_preview.to_owned(),
        byte_size,
        badges,
    }
}

fn subscription(topic_filter: &str, qos: QosLevel, message_count: usize) -> SubscriptionRow {
    SubscriptionRow {
        topic_filter: topic_filter.to_owned(),
        qos,
        message_count,
        active: true,
        messages_visible: true,
        selected: false,
    }
}

fn message(
    id: u32,
    topic: &str,
    timestamp: &str,
    qos: QosLevel,
    retained: bool,
    payload_preview: &str,
    byte_size: usize,
    badges: &[&str],
) -> MessageRow {
    MessageRow {
        id,
        topic: topic.to_owned(),
        timestamp: timestamp.to_owned(),
        qos,
        retained,
        payload: payload_preview.as_bytes().to_vec(),
        payload_preview: payload_preview.to_owned(),
        byte_size,
        badges: badges.iter().map(|badge| (*badge).to_owned()).collect(),
        diagnostics: Vec::new(),
        formatted_detail: None,
    }
}
