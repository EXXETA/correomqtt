use std::path::PathBuf;

use correo_mqtt::ConnectionId;
use thiserror::Error;

use crate::{
    BuiltInBrokerEvent, ConnectDisabledReason, ConnectionPluginDirection,
    ConnectionPluginWorkflowField, ConnectionSecretField, ConnectionSettingField,
    ConnectionSettingFlag, ConnectionSettingsSnapshot, ConnectionSettingsTab, ConnectionState,
    ConnectionSummary, Diagnostic, GlobalSettingField, GlobalSettingFlag, GlobalSettingsSnapshot,
    MessageInspectorTab, MigrationApplyStage, MigrationRecoveryCompletion, MigrationRecoveryCounts,
    MigrationRecoveryDiagnostic, MigrationRecoveryFailure, MigrationRecoveryRow,
    MigrationRecoveryWarning, MqttCommand, MqttEvent, PluginHookKind, PluginSurfaceTab,
    PluginWorkflowEvent, QosLevel, ScriptDetailTab, ScriptExecutionError, ScriptExecutionStatus,
    ScriptLogLevel, SecretInput, SettingsSection, StartupState, ThemeMode, TransferSection,
    TransferStep, WorkbenchTab, Workspace,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppCommand {
    SelectWorkspace(Workspace),
    StartBuiltInBroker,
    StopBuiltInBroker,
    UpdateBuiltInBrokerPort(String),
    SetBuiltInBrokerCredentialsEnabled(bool),
    UpdateBuiltInBrokerUsername(String),
    UpdateBuiltInBrokerPassword(SecretInput),
    ClearBuiltInBrokerLogs,
    SetThemeMode(ThemeMode),
    SearchConnections(String),
    SelectConnection(ConnectionId),
    MoveConnection {
        connection_id: ConnectionId,
        target_connection_id: ConnectionId,
        after: bool,
    },
    OpenConnectionLauncher,
    OpenConnectionWorkbench(ConnectionId),
    OpenConnectionSettings(ConnectionId),
    Connect(ConnectionId),
    EditConnection(ConnectionId),
    Reconnect(ConnectionId),
    Disconnect(ConnectionId),
    DuplicateConnection(ConnectionId),
    AddConnection,
    ImportConnections,
    ExportConnections,
    ChooseConnectionImportFile(PathBuf),
    SubmitConnectionImportPassword(String),
    ClearConnectionImportError,
    SelectConnectionImportRow {
        row_id: String,
        selected: bool,
    },
    StartConnectionImport,
    SelectConnectionExportRow {
        row_id: String,
        selected: bool,
    },
    SetConnectionExportEncrypted(bool),
    UpdateConnectionExportPath(String),
    StartConnectionExport,
    ImportMessages,
    ImportMessagesFromPath(PathBuf),
    ExportMessages,
    ExportPublishHistoryMessage(u32),
    ExportIncomingMessage(u32),
    ExportPublishHistoryMessageToPath {
        message_id: u32,
        path: PathBuf,
    },
    ExportIncomingMessageToPath {
        message_id: u32,
        path: PathBuf,
    },
    CopyPublishHistoryMessageToPublishForm(u32),
    CopyIncomingMessageToPublishForm(u32),
    RemovePublishHistoryMessage(u32),
    RemoveIncomingMessage(u32),
    ClearPublishHistory,
    ClearIncomingMessages,
    MigrationRecovery(MigrationRecoveryCommand),
    SelectWorkbenchTab(WorkbenchTab),
    UpdatePublishTopic(String),
    UpdatePublishPayload(String),
    UpdatePublishQos(QosLevel),
    SetPublishRetained(bool),
    SearchPublishHistory(String),
    SelectPublishHistoryMessage(u32),
    Publish,
    UpdateSubscribeTopic(String),
    UpdateSubscribeQos(QosLevel),
    Subscribe,
    Unsubscribe(String),
    UnsubscribeAll,
    CancelUnsubscribeAll,
    ConfirmUnsubscribeAll,
    SetSubscriptionMessagesVisible {
        topic_filter: String,
        visible: bool,
    },
    SetAllSubscriptionMessagesVisible(bool),
    SelectSubscription {
        topic_filter: String,
        extend: bool,
        toggle: bool,
    },
    SearchMessages(String),
    SelectMessage(u32),
    SelectInspectorTab(MessageInspectorTab),
    SelectDetailTransform(Option<String>),
    SelectDetailFormatter(Option<String>),
    RefreshMessageDetail,
    SelectConnectionSettingsTab(ConnectionSettingsTab),
    UpdateConnectionSetting {
        field: ConnectionSettingField,
        value: String,
    },
    UpdateConnectionSecret {
        field: ConnectionSecretField,
        value: SecretInput,
    },
    SetConnectionSettingFlag {
        flag: ConnectionSettingFlag,
        enabled: bool,
    },
    GenerateClientId,
    SetLwtEnabled(bool),
    SaveConnectionSettings,
    DiscardConnectionSettings,
    OpenConnectionPlugins(ConnectionId),
    SaveConnectionPlugins,
    CloseConnectionPlugins,
    SelectConnectionPluginWorkflow(usize),
    AddConnectionPluginWorkflow {
        plugin_id: String,
    },
    RemoveConnectionPluginWorkflow {
        index: usize,
    },
    SetConnectionPluginWorkflowEnabled {
        index: usize,
        enabled: bool,
    },
    MoveConnectionPluginWorkflow {
        index: usize,
        target_index: usize,
        after: bool,
    },
    SetConnectionPluginWorkflowDirection {
        index: usize,
        direction: ConnectionPluginDirection,
    },
    UpdateConnectionPluginWorkflowField {
        index: usize,
        field: ConnectionPluginWorkflowField,
        value: String,
    },
    RequestDeleteConnection,
    CancelDeleteConnection,
    ConfirmDeleteConnection,
    SelectTransferSection(TransferSection),
    SelectTransferStep(TransferStep),
    SelectGlobalSettingsSection(SettingsSection),
    UpdateGlobalSetting {
        field: GlobalSettingField,
        value: String,
    },
    SetGlobalSettingFlag {
        flag: GlobalSettingFlag,
        enabled: bool,
    },
    AddPluginRepository,
    UpdatePluginRepository {
        index: usize,
        url: String,
    },
    RemovePluginRepository {
        index: usize,
    },
    SaveGlobalSettings,
    DiscardGlobalSettings,
    SearchScripts(String),
    SelectScriptConnection(String),
    SelectScript(String),
    RequestCreateScript,
    UpdateNewScriptName(String),
    CancelCreateScript,
    CreateScript,
    UpdateScriptSource(String),
    SaveScript,
    DiscardScriptChanges,
    RequestRenameScript,
    UpdateRenameScriptName(String),
    CancelRenameScript,
    ConfirmRenameScript,
    RequestDeleteScript,
    CancelDeleteScript,
    ConfirmDeleteScript,
    SelectScriptDetailTab(ScriptDetailTab),
    SelectScriptExecution(String),
    RunScript,
    CancelScript,
    RemoveScriptExecution(String),
    ClearFinishedScriptExecutions,
    SearchPlugins(String),
    SelectPlugin(String),
    SelectMarketplacePlugin(String),
    SelectPluginSurfaceTab(PluginSurfaceTab),
    InstallMarketplacePlugin {
        marketplace_plugin_id: String,
    },
    UninstallPlugin {
        plugin_id: String,
    },
    SetPluginEnabled {
        plugin_id: String,
        enabled: bool,
    },
    UpdatePluginConfigValue {
        plugin_id: String,
        key: String,
        value: String,
    },
    ApplyPluginConfig {
        plugin_id: String,
    },
    CancelPluginConfig {
        plugin_id: String,
    },
    ResetPluginConfig {
        plugin_id: String,
    },
    SetPluginHookEnabled {
        plugin_id: String,
        hook: PluginHookKind,
        enabled: bool,
    },
    CancelPluginDisable,
    ConfirmPluginDisable,
    StartAddPluginHook {
        plugin_id: String,
    },
    StartEditPluginHook {
        plugin_id: String,
        hook: PluginHookKind,
    },
    SetPluginHookDraftEnabled(bool),
    UpdatePluginHookTarget(String),
    UpdatePluginHookConfigJson(String),
    ApplyPluginHookEdit,
    CancelPluginHookEdit,
    ResetPluginHookEdit,
    SearchPluginDiagnostics(String),
    SelectPluginDiagnostic(String),
    ClearPluginDiagnostics,
    InvokeConnectionPluginAction {
        plugin_id: String,
        action_id: String,
        connection_id: ConnectionId,
    },
    ClosePluginWindow {
        plugin_id: String,
        action_id: String,
        connection_id: ConnectionId,
    },
    Mqtt(MqttCommand),
    Shutdown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationRecoveryCommand {
    ChooseMigrate,
    StartEmptyProfile,
    CancelEmptyProfile,
    ConfirmStartEmptyProfile,
    SubmitPassword,
    SkipSecrets,
    SelectMigrationItem { item_id: String, selected: bool },
    ApplyMigration,
    Retry,
    RequestRestoreBackup,
    CancelRestoreBackup,
    ConfirmRestoreBackup,
    OpenDiagnostics,
    OpenSettingsData,
    OpenConnections,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppEvent {
    ConnectionListLoaded {
        connections: Vec<ConnectionSummary>,
    },
    ConnectionOpened {
        connection_id: ConnectionId,
    },
    ConnectionClosed {
        connection_id: ConnectionId,
    },
    ConnectionStateChanged {
        connection_id: ConnectionId,
        state: ConnectionState,
        disabled_reason: Option<ConnectDisabledReason>,
        last_activity: String,
    },
    ConnectionSettingsLoaded {
        connection_id: ConnectionId,
        settings: ConnectionSettingsSnapshot,
    },
    GlobalSettingsLoaded {
        settings: GlobalSettingsSnapshot,
    },
    ThemeModeChanged {
        mode: ThemeMode,
    },
    MigrationApplied {
        state: Box<StartupState>,
        completion: MigrationRecoveryCompletion,
        diagnostics: Vec<MigrationRecoveryDiagnostic>,
    },
    DiagnosticRaised(Diagnostic),
    ScriptExecutionLogAppended {
        execution_id: String,
        level: ScriptLogLevel,
        message: String,
        timestamp: String,
    },
    ScriptExecutionUpdated {
        execution_id: String,
        status: ScriptExecutionStatus,
        duration: String,
        error: Option<ScriptExecutionError>,
    },
    MigrationRecovery(MigrationRecoveryEvent),
    Mqtt(MqttEvent),
    BuiltInBroker(BuiltInBrokerEvent),
    PluginWorkflow(PluginWorkflowEvent),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationRecoveryEvent {
    NoLegacyData,
    LegacyDetected {
        legacy_path: String,
        counts: MigrationRecoveryCounts,
        warnings: Vec<MigrationRecoveryWarning>,
    },
    DetectionFailed {
        message: String,
    },
    BackupStarted,
    BackupCreated {
        backup_name: String,
        backup_path_hint: String,
    },
    BackupFailed {
        message: String,
    },
    PasswordNeeded,
    PasswordRejected,
    UnsupportedEncryption,
    SecretsUnlocked {
        imported_count: usize,
    },
    SecretsSkipped {
        skipped_count: usize,
    },
    ReviewReady {
        counts: MigrationRecoveryCounts,
        rows: Vec<MigrationRecoveryRow>,
        warnings: Vec<MigrationRecoveryWarning>,
    },
    ApplyStarted,
    ApplyStageChanged {
        stage: MigrationApplyStage,
    },
    ApplyCompleted {
        completion: MigrationRecoveryCompletion,
        diagnostics: Vec<MigrationRecoveryDiagnostic>,
    },
    ApplyFailed {
        failure: MigrationRecoveryFailure,
    },
    RestoreStarted,
    RestoreCompleted,
    RestoreFailed {
        message: String,
    },
}

#[derive(Debug, Clone)]
pub struct AppCommandSender {
    sender: flume::Sender<AppCommand>,
}

impl AppCommandSender {
    pub(crate) fn new(sender: flume::Sender<AppCommand>) -> Self {
        Self { sender }
    }

    pub fn disconnected() -> Self {
        let (sender, receiver) = flume::bounded(0);
        drop(receiver);
        Self { sender }
    }

    pub fn send(&self, command: AppCommand) -> Result<(), CommandSendError> {
        self.sender.try_send(command).map_err(|error| match error {
            flume::TrySendError::Full(command) => CommandSendError::CommandFull(Box::new(command)),
            flume::TrySendError::Disconnected(command) => {
                CommandSendError::CommandDisconnected(Box::new(command))
            }
        })
    }

    pub fn push(&mut self, command: AppCommand) -> Result<(), CommandSendError> {
        self.send(command)
    }
}

#[derive(Debug, Clone)]
pub struct AppEventSender {
    sender: flume::Sender<AppEvent>,
}

impl AppEventSender {
    pub(crate) fn new(sender: flume::Sender<AppEvent>) -> Self {
        Self { sender }
    }

    pub fn emit(&self, event: AppEvent) -> Result<(), CommandSendError> {
        self.sender.try_send(event).map_err(|error| match error {
            flume::TrySendError::Full(event) => CommandSendError::EventFull(Box::new(event)),
            flume::TrySendError::Disconnected(event) => {
                CommandSendError::EventDisconnected(Box::new(event))
            }
        })
    }
}

#[derive(Debug, Error)]
pub enum CommandSendError {
    #[error("app command queue is full")]
    CommandFull(Box<AppCommand>),
    #[error("app command receiver is disconnected")]
    CommandDisconnected(Box<AppCommand>),
    #[error("app event queue is full")]
    EventFull(Box<AppEvent>),
    #[error("app event receiver is disconnected")]
    EventDisconnected(Box<AppEvent>),
}
