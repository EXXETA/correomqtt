use correo_core::{
    sample_snapshot, ConnectionSurface, ExportPasswordConfirmation, ExportPathState,
    ImportPasswordState, PluginDisableConfirmation, PluginHookDraft, PluginHookEditor,
    PluginHookKind, PluginLoadState, PluginSurfaceTab, SettingsSection, ThemeMode,
    TransferFeedback, TransferOutcome, TransferSection, TransferStep, WorkbenchTab, Workspace,
};
pub(super) const REQUIRED_SIZES: [(u32, u32); 3] = [(1280, 800), (1024, 768), (900, 640)];
const NARROW_WORKBENCH_SIZE: (u32, u32) = (720, 640);
const PLUGIN_MANAGER_SIZES: [(u32, u32); 3] = [(1280, 800), (1024, 700), (900, 600)];
const PLUGIN_STATE_SIZE: (u32, u32) = (1024, 700);
const TRANSFER_STATE_SIZE: (u32, u32) = (900, 640);
const SETTINGS_SECTION_SIZE: (u32, u32) = (900, 640);
const REQUIRED_MODES: [ThemeMode; 2] = [ThemeMode::Light, ThemeMode::Dark];

const FULL_MATRIX_SCENARIOS: [Scenario; 2] = [Scenario::ConnectionsEmpty, Scenario::Workbench];
const SECONDARY_SCENARIOS: [Scenario; 6] = [
    Scenario::Settings,
    Scenario::Scripts,
    Scenario::ImportExport,
    Scenario::Plugins,
    Scenario::Diagnostics,
    Scenario::GlobalSettings,
];
const PLUGIN_STATE_SCENARIOS: [Scenario; 6] = [
    Scenario::PluginsLoading,
    Scenario::PluginsEmpty,
    Scenario::PluginsDisableConfirm,
    Scenario::PluginsLoadError,
    Scenario::PluginsHookConfigInvalid,
    Scenario::PluginsDiagnosticsFiltered,
];
const TRANSFER_STATE_SCENARIOS: [Scenario; 13] = [
    Scenario::ImportChooseFile,
    Scenario::ImportPasswordNeeded,
    Scenario::ImportPasswordError,
    Scenario::ImportReviewWarnings,
    Scenario::ImportCompleteSuccess,
    Scenario::ImportCompleteFailure,
    Scenario::ExportPlain,
    Scenario::ExportEncrypted,
    Scenario::ExportMissingExtension,
    Scenario::ExportInvalidPath,
    Scenario::ExportSuccess,
    Scenario::ExportFailure,
    Scenario::MessageTransfer,
];
const SETTINGS_STATE_SCENARIOS: [Scenario; 3] = [
    Scenario::GlobalSettingsLanguage,
    Scenario::GlobalSettingsSearch,
    Scenario::GlobalSettingsKeyring,
];
const CONNECTION_STATE_SCENARIOS: [Scenario; 1] = [Scenario::SettingsOverlay];
#[derive(Clone)]
pub(super) struct Capture {
    pub(super) scenario: Scenario,
    pub(super) mode: ThemeMode,
    pub(super) size: (u32, u32),
    pub(super) zoom_factor: f32,
    pub(super) file_name: String,
}

impl Capture {
    fn new(scenario: Scenario, mode: ThemeMode, size: (u32, u32)) -> Self {
        let mode_slug = mode_slug(&mode);
        Self {
            scenario,
            mode,
            size,
            zoom_factor: 1.0,
            file_name: format!(
                "correo-{}-{}-{}x{}.png",
                scenario.slug(),
                mode_slug,
                size.0,
                size.1
            ),
        }
    }

    fn zoomed(mut self, zoom_factor: f32) -> Self {
        self.zoom_factor = zoom_factor;
        self.file_name = format!(
            "correo-{}-{}-{}x{}-zoom-{}pct.png",
            self.scenario.slug(),
            mode_slug(&self.mode),
            self.size.0,
            self.size.1,
            (zoom_factor * 100.0).round() as u32,
        );
        self
    }
}
#[derive(Clone, Copy)]
pub(super) enum Scenario {
    ConnectionsEmpty,
    Workbench,
    Settings,
    SettingsOverlay,
    Scripts,
    ImportExport,
    Plugins,
    Diagnostics,
    GlobalSettings,
    PluginsLoading,
    PluginsEmpty,
    PluginsDisableConfirm,
    PluginsLoadError,
    PluginsHookConfigInvalid,
    PluginsDiagnosticsFiltered,
    ImportChooseFile,
    ImportPasswordNeeded,
    ImportPasswordError,
    ImportReviewWarnings,
    ImportCompleteSuccess,
    ImportCompleteFailure,
    ExportPlain,
    ExportEncrypted,
    ExportMissingExtension,
    ExportInvalidPath,
    ExportSuccess,
    ExportFailure,
    MessageTransfer,
    GlobalSettingsLanguage,
    GlobalSettingsSearch,
    GlobalSettingsKeyring,
}

impl Scenario {
    fn slug(self) -> &'static str {
        match self {
            Self::ConnectionsEmpty => "connections-empty",
            Self::Workbench => "workbench",
            Self::Settings => "connection-settings",
            Self::SettingsOverlay => "connection-settings-overlay",
            Self::Scripts => "scripts",
            Self::ImportExport => "import-export",
            Self::Plugins => "plugins",
            Self::Diagnostics => "diagnostics",
            Self::GlobalSettings => "global-settings-appearance",
            Self::PluginsLoading => "plugins-loading",
            Self::PluginsEmpty => "plugins-empty",
            Self::PluginsDisableConfirm => "plugins-disable-confirm",
            Self::PluginsLoadError => "plugins-load-error",
            Self::PluginsHookConfigInvalid => "plugins-hook-config-invalid",
            Self::PluginsDiagnosticsFiltered => "plugins-diagnostics-filtered",
            Self::ImportChooseFile => "import-cqc-choose-file",
            Self::ImportPasswordNeeded => "import-cqc-password-needed",
            Self::ImportPasswordError => "import-cqc-password-error",
            Self::ImportReviewWarnings => "import-cqc-review-warnings",
            Self::ImportCompleteSuccess => "import-cqc-complete-success",
            Self::ImportCompleteFailure => "import-cqc-complete-failure",
            Self::ExportPlain => "export-cqc-plain",
            Self::ExportEncrypted => "export-cqc-encrypted",
            Self::ExportMissingExtension => "export-cqc-missing-extension",
            Self::ExportInvalidPath => "export-cqc-invalid-path",
            Self::ExportSuccess => "export-cqc-success",
            Self::ExportFailure => "export-cqc-failure",
            Self::MessageTransfer => "message-cqm-actions",
            Self::GlobalSettingsLanguage => "global-settings-language",
            Self::GlobalSettingsSearch => "global-settings-search",
            Self::GlobalSettingsKeyring => "global-settings-keyring",
        }
    }

    pub(super) fn label(self) -> &'static str {
        match self {
            Self::ConnectionsEmpty => "empty connections",
            Self::Workbench => "active workbench",
            Self::Settings => "connection settings",
            Self::SettingsOverlay => "connection settings modal overlay",
            Self::Scripts => "scripts",
            Self::ImportExport => "import/export",
            Self::Plugins => "plugins",
            Self::Diagnostics => "diagnostics",
            Self::GlobalSettings => "global settings appearance",
            Self::PluginsLoading => "plugins loading",
            Self::PluginsEmpty => "plugins empty",
            Self::PluginsDisableConfirm => "plugins disable confirmation",
            Self::PluginsLoadError => "plugins WASM load error",
            Self::PluginsHookConfigInvalid => "plugins hook config validation",
            Self::PluginsDiagnosticsFiltered => "plugins filtered diagnostics",
            Self::ImportChooseFile => ".cqc import choose file",
            Self::ImportPasswordNeeded => ".cqc import password needed",
            Self::ImportPasswordError => ".cqc import recoverable password error",
            Self::ImportReviewWarnings => ".cqc import review warnings",
            Self::ImportCompleteSuccess => ".cqc import success outcome",
            Self::ImportCompleteFailure => ".cqc import failure outcome",
            Self::ExportPlain => ".cqc export plain",
            Self::ExportEncrypted => ".cqc export encrypted",
            Self::ExportMissingExtension => ".cqc export missing extension",
            Self::ExportInvalidPath => ".cqc export invalid path",
            Self::ExportSuccess => ".cqc export success outcome",
            Self::ExportFailure => ".cqc export failure outcome",
            Self::MessageTransfer => "message .cqm entry points",
            Self::GlobalSettingsLanguage => "global settings language",
            Self::GlobalSettingsSearch => "global settings search",
            Self::GlobalSettingsKeyring => "global settings keyring",
        }
    }

    fn representative_size(self) -> (u32, u32) {
        match self {
            Self::Scripts => (1280, 800),
            Self::Plugins
            | Self::PluginsLoading
            | Self::PluginsEmpty
            | Self::PluginsDisableConfirm
            | Self::PluginsLoadError
            | Self::PluginsHookConfigInvalid
            | Self::PluginsDiagnosticsFiltered => PLUGIN_STATE_SIZE,
            Self::GlobalSettings
            | Self::GlobalSettingsLanguage
            | Self::GlobalSettingsSearch
            | Self::GlobalSettingsKeyring => SETTINGS_SECTION_SIZE,
            Self::ImportChooseFile
            | Self::ImportPasswordNeeded
            | Self::ImportPasswordError
            | Self::ImportReviewWarnings
            | Self::ImportCompleteSuccess
            | Self::ImportCompleteFailure
            | Self::ExportPlain
            | Self::ExportEncrypted
            | Self::ExportMissingExtension
            | Self::ExportInvalidPath
            | Self::ExportSuccess
            | Self::ExportFailure
            | Self::MessageTransfer => TRANSFER_STATE_SIZE,
            _ => (1024, 768),
        }
    }
}

pub(super) fn screenshot_captures() -> Vec<Capture> {
    let mut captures = Vec::new();
    for scenario in FULL_MATRIX_SCENARIOS {
        for mode in REQUIRED_MODES {
            for size in REQUIRED_SIZES {
                captures.push(Capture::new(scenario, mode.clone(), size));
            }
        }
    }
    for mode in REQUIRED_MODES {
        captures.push(Capture::new(
            Scenario::Workbench,
            mode.clone(),
            NARROW_WORKBENCH_SIZE,
        ));
    }
    captures.push(
        Capture::new(Scenario::Workbench, ThemeMode::Light, (1024, 768)).zoomed(1.5),
    );
    for scenario in SECONDARY_SCENARIOS {
        for mode in REQUIRED_MODES {
            if matches!(scenario, Scenario::Plugins) {
                for size in PLUGIN_MANAGER_SIZES {
                    captures.push(Capture::new(scenario, mode.clone(), size));
                }
            } else {
                captures.push(Capture::new(
                    scenario,
                    mode.clone(),
                    scenario.representative_size(),
                ));
            }
        }
    }
    for scenario in PLUGIN_STATE_SCENARIOS {
        captures.push(Capture::new(
            scenario,
            ThemeMode::Light,
            scenario.representative_size(),
        ));
    }
    for scenario in TRANSFER_STATE_SCENARIOS {
        captures.push(Capture::new(
            scenario,
            ThemeMode::Light,
            scenario.representative_size(),
        ));
    }
    for scenario in SETTINGS_STATE_SCENARIOS {
        captures.push(Capture::new(
            scenario,
            ThemeMode::Light,
            scenario.representative_size(),
        ));
    }
    for scenario in CONNECTION_STATE_SCENARIOS {
        captures.push(Capture::new(
            scenario,
            ThemeMode::Light,
            scenario.representative_size(),
        ));
    }
    captures
}

pub(super) fn snapshot_for(capture: &Capture) -> correo_core::AppSnapshot {
    let mut snapshot = sample_snapshot(capture.mode.clone());
    snapshot.active_workspace = workspace_for(capture.scenario);
    let connection_transfer = transfer_section_for(capture.scenario, TransferSection::Messages)
        != TransferSection::Messages;
    snapshot.connection_surface = match capture.scenario {
        Scenario::ConnectionsEmpty => ConnectionSurface::Workbench,
        Scenario::Settings => ConnectionSurface::Settings,
        Scenario::SettingsOverlay => ConnectionSurface::Workbench,
        _ if connection_transfer => ConnectionSurface::Transfer,
        _ => ConnectionSurface::Workbench,
    };
    if matches!(capture.scenario, Scenario::SettingsOverlay) {
        snapshot.connection_settings_overlay = snapshot.selected_connection;
    }
    if matches!(capture.scenario, Scenario::ConnectionsEmpty) {
        snapshot.active_connection = None;
        snapshot.connection_count = 0;
        snapshot.connections.clear();
        snapshot.selected_connection = None;
    }
    if matches!(capture.scenario, Scenario::Workbench) && capture.size.0 <= 1024 {
        snapshot.workbench.narrow_tab = WorkbenchTab::Subscribe;
    }
    apply_plugin_scenario(capture.scenario, &mut snapshot);
    apply_transfer_scenario(capture.scenario, &mut snapshot);
    apply_settings_scenario(capture.scenario, &mut snapshot);
    snapshot
}

pub(super) fn mode_slug(mode: &ThemeMode) -> &'static str {
    if matches!(mode, ThemeMode::System) {
        "system"
    } else if mode.is_light() {
        "light"
    } else {
        "dark"
    }
}

fn workspace_for(scenario: Scenario) -> Workspace {
    match scenario {
        Scenario::ConnectionsEmpty
        | Scenario::Workbench
        | Scenario::Settings
        | Scenario::SettingsOverlay
        | Scenario::MessageTransfer => Workspace::Connections,
        Scenario::Scripts => Workspace::Scripts,
        Scenario::Plugins
        | Scenario::PluginsLoading
        | Scenario::PluginsEmpty
        | Scenario::PluginsDisableConfirm
        | Scenario::PluginsLoadError
        | Scenario::PluginsHookConfigInvalid
        | Scenario::PluginsDiagnosticsFiltered => Workspace::Plugins,
        Scenario::Diagnostics => Workspace::Diagnostics,
        Scenario::GlobalSettings
        | Scenario::GlobalSettingsLanguage
        | Scenario::GlobalSettingsSearch
        | Scenario::GlobalSettingsKeyring => Workspace::Settings,
        _ => Workspace::Connections,
    }
}

fn apply_transfer_scenario(scenario: Scenario, snapshot: &mut correo_core::AppSnapshot) {
    snapshot.transfer.active_section =
        transfer_section_for(scenario, snapshot.transfer.active_section);
    match scenario {
        Scenario::ImportChooseFile => {
            snapshot.transfer.active_step = TransferStep::ChooseFile;
            snapshot.transfer.import.file = None;
            snapshot.transfer.import.feedback = None;
        }
        Scenario::ImportPasswordNeeded => {
            snapshot.transfer.active_step = TransferStep::Password;
            snapshot.transfer.import.password_state = ImportPasswordState::Needed;
            snapshot.transfer.import.feedback = Some(TransferFeedback::info(
                "Encrypted file detected; enter the export password to continue.",
            ));
        }
        Scenario::ImportPasswordError => {
            snapshot.transfer.active_step = TransferStep::Password;
            snapshot.transfer.import.password_state = ImportPasswordState::InvalidRecoverable;
            snapshot.transfer.import.feedback = Some(TransferFeedback::error(
                "Password did not unlock this .cqc file.",
            ));
        }
        Scenario::ImportReviewWarnings => {
            snapshot.transfer.active_step = TransferStep::Review;
        }
        Scenario::ImportCompleteSuccess => {
            snapshot.transfer.active_step = TransferStep::Complete;
            snapshot.transfer.import.outcome = Some(TransferOutcome::success(
                "Import complete",
                "2 connection profiles imported; secrets remain in the OS keyring.",
            ));
        }
        Scenario::ImportCompleteFailure => {
            snapshot.transfer.active_step = TransferStep::Complete;
            snapshot.transfer.import.outcome = Some(TransferOutcome::failure(
                "Import failed",
                "The selected file was valid, but no importable profiles remained after review.",
            ));
        }
        Scenario::ExportPlain => {
            snapshot.transfer.export.encrypted = false;
            snapshot.transfer.export.password_confirmation =
                ExportPasswordConfirmation::NotRequired;
            snapshot.transfer.export.feedback = Some(TransferFeedback::warning(
                "Plain export excludes sensitive auth values.",
            ));
        }
        Scenario::ExportEncrypted => {
            snapshot.transfer.export.encrypted = true;
            snapshot.transfer.export.password_confirmation = ExportPasswordConfirmation::Needed;
            snapshot.transfer.export.feedback = Some(TransferFeedback::info(
                "Encrypted export requires a password and confirmation before writing.",
            ));
        }
        Scenario::ExportMissingExtension => {
            snapshot.transfer.export.output_path = "Exports/correomqtt-connections".to_owned();
            snapshot.transfer.export.path_state = ExportPathState::MissingExtension;
            snapshot.transfer.export.feedback = Some(TransferFeedback::warning(
                "The target file should end with .cqc.",
            ));
        }
        Scenario::ExportInvalidPath => {
            snapshot.transfer.export.output_path = String::new();
            snapshot.transfer.export.path_state = ExportPathState::InvalidPath;
            snapshot.transfer.export.feedback = Some(TransferFeedback::error(
                "Choose a writable target path before exporting.",
            ));
        }
        Scenario::ExportSuccess => {
            snapshot.transfer.export.outcome = Some(TransferOutcome::success(
                "Export complete",
                "3 encrypted connection profiles exported.",
            ));
        }
        Scenario::ExportFailure => {
            snapshot.transfer.export.outcome = Some(TransferOutcome::failure(
                "Export failed",
                "The target folder was unavailable when export started.",
            ));
        }
        Scenario::MessageTransfer => {
            snapshot.transfer.messages.feedback = Some(TransferFeedback::info(
                "Message .cqm files load into publish and export from message rows.",
            ));
        }
        _ => {}
    }
}

