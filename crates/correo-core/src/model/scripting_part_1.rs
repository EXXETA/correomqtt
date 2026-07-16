use crate::{
    redact_sensitive, Diagnostic, ScriptDetailTab, ScriptExecutionError, ScriptExecutionErrorKind,
    ScriptExecutionRow, ScriptExecutionStatus, ScriptFeedback, ScriptFeedbackSeverity,
    ScriptFileStatus, ScriptLogLevel, ScriptLogLine, ScriptRow,
};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use super::AppModel;

