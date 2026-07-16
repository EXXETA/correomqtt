fn review_rows(preview: &MigrationPreview) -> Vec<MigrationRecoveryRow> {
    let mut rows = MigrationRecoverySnapshot::review_rows();
    for row in &mut rows {
        row.detail = match row.task {
            MigrationRecoveryTask::Connections => {
                format!("{} profile(s) ready to migrate.", preview.connections.len())
            }
            MigrationRecoveryTask::Histories => {
                format!(
                    "{} connection history set(s) ready.",
                    preview.histories.connections.len()
                )
            }
            MigrationRecoveryTask::Scripts => {
                format!(
                    "{} script file(s) and metadata ready.",
                    preview.scripts.files.len()
                )
            }
            MigrationRecoveryTask::Plugins => {
                MigrationRecoverySnapshot::plugin_replacement_body().to_owned()
            }
            _ => row.detail.clone(),
        };
    }
    rows
}

fn preview_warnings(warnings: &[MigrationWarning]) -> Vec<MigrationRecoveryWarning> {
    warnings
        .iter()
        .map(|warning| {
            MigrationRecoveryWarning::new(
                warning_kind(warning.code),
                redact_sensitive(&warning.message),
            )
        })
        .collect()
}

fn warning_kind(code: &str) -> MigrationRecoveryWarningKind {
    match code {
        "unsupported_legacy_field" => MigrationRecoveryWarningKind::UnsupportedLegacyField,
        "legacy_hooks_not_mapped" | "legacy_hooks_partially_mapped" | "legacy_hook_not_mapped" => {
            MigrationRecoveryWarningKind::HookConfigIgnored
        }
        "legacy_plugins_ignored" => MigrationRecoveryWarningKind::JavaPluginStateIgnored,
        _ => MigrationRecoveryWarningKind::ConnectionNeedsReview,
    }
}

fn completion_from_diagnostics(diagnostics: &MigrationDiagnostics) -> MigrationRecoveryCompletion {
    if diagnostics.warnings.is_empty() && diagnostics.unmapped_fields.is_empty() {
        MigrationRecoveryCompletion::Success
    } else {
        MigrationRecoveryCompletion::PartialSuccess
    }
}

fn recovery_diagnostics(diagnostics: &MigrationDiagnostics) -> Vec<MigrationRecoveryDiagnostic> {
    let mut recovery = Vec::new();
    recovery.extend(diagnostics.warnings.iter().map(|warning| {
        MigrationRecoveryDiagnostic::warning(
            MigrationDiagnosticCategory::Migration,
            format!("{}: {}", warning.code, warning.message),
        )
    }));
    recovery.extend(diagnostics.unmapped_fields.iter().map(|field| {
        MigrationRecoveryDiagnostic::warning(
            MigrationDiagnosticCategory::LegacyField,
            format!("Unsupported legacy field ignored: {field}"),
        )
    }));
    recovery.extend(diagnostics.recovery_steps.iter().map(|step| {
        MigrationRecoveryDiagnostic::info(MigrationDiagnosticCategory::Backup, step.clone())
    }));
    recovery
}
