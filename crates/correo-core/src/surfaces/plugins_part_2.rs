#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginDiagnosticRow {
    pub id: String,
    pub plugin_id: String,
    pub severity: PluginDiagnosticSeverity,
    pub hook: Option<PluginHookKind>,
    pub message: String,
    pub detail: String,
    pub occurred_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

impl PluginDiagnosticSeverity {
    pub fn label(self) -> &'static str {
        match self {
            Self::Info => "Info",
            Self::Warning => "Warning",
            Self::Error => "Error",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginFeedback {
    pub severity: PluginFeedbackSeverity,
    pub message: String,
}

impl PluginFeedback {
    pub fn info(message: impl Into<String>) -> Self {
        Self {
            severity: PluginFeedbackSeverity::Info,
            message: message.into(),
        }
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            severity: PluginFeedbackSeverity::Warning,
            message: message.into(),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            severity: PluginFeedbackSeverity::Error,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginFeedbackSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginDisableConfirmation {
    pub plugin_id: String,
    pub plugin_name: String,
    pub active_hooks: Vec<PluginHookKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginHookEditor {
    pub plugin_id: String,
    pub plugin_name: String,
    pub original: Option<PluginHookDraft>,
    pub draft: PluginHookDraft,
    pub error: Option<String>,
}

impl PluginHookEditor {
    pub fn is_new(&self) -> bool {
        self.original.is_none()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginHookDraft {
    pub hook: PluginHookKind,
    pub enabled: bool,
    pub target: String,
    pub config_json: String,
}

impl From<&PluginHookAssignment> for PluginHookDraft {
    fn from(assignment: &PluginHookAssignment) -> Self {
        Self {
            hook: assignment.hook,
            enabled: assignment.enabled,
            target: assignment.target.clone(),
            config_json: assignment.config_json.clone(),
        }
    }
}
