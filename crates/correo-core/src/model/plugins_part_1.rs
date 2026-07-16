use crate::{
    AppCommand, Diagnostic, PluginDisableConfirmation, PluginFeedback, PluginHookDraft,
    PluginHookEditor, PluginHookKind, PluginHookSettingsSnapshot, PluginHookStatus,
    PluginStateSnapshot, PluginStatus, PluginSurfaceTab,
};

use super::AppModel;

#[path = "plugins/helpers.rs"]
mod helpers;
#[path = "plugins/manager.rs"]
mod manager;
use helpers::*;

impl AppModel {
    pub(super) fn apply_plugin_command(&mut self, command: &AppCommand) -> bool {
        match command {
            AppCommand::SearchPlugins(filter) => self.search_plugins(filter.clone()),
            AppCommand::SelectPlugin(plugin_id) => self.select_plugin(plugin_id.clone()),
            AppCommand::SelectMarketplacePlugin(plugin_id) => {
                self.select_marketplace_plugin(plugin_id.clone())
            }
            AppCommand::SelectPluginSurfaceTab(tab) => self.select_plugin_surface_tab(*tab),
            AppCommand::InstallMarketplacePlugin {
                marketplace_plugin_id,
            } => self.install_marketplace_plugin(marketplace_plugin_id.clone()),
            AppCommand::UninstallPlugin { plugin_id } => self.uninstall_plugin(plugin_id.clone()),
            AppCommand::SetPluginEnabled { plugin_id, enabled } => {
                self.set_plugin_enabled(plugin_id.clone(), *enabled);
            }
            AppCommand::UpdatePluginConfigValue {
                plugin_id,
                key,
                value,
            } => self.update_plugin_config_value(plugin_id.clone(), key.clone(), value.clone()),
            AppCommand::ApplyPluginConfig { plugin_id } => {
                self.apply_plugin_config(plugin_id.clone());
            }
            AppCommand::CancelPluginConfig { plugin_id } => {
                self.cancel_plugin_config(plugin_id.clone());
            }
            AppCommand::ResetPluginConfig { plugin_id } => {
                self.reset_plugin_config(plugin_id.clone());
            }
            AppCommand::SetPluginHookEnabled {
                plugin_id,
                hook,
                enabled,
            } => self.set_plugin_hook_enabled(plugin_id.clone(), *hook, *enabled),
            AppCommand::CancelPluginDisable => self.cancel_plugin_disable(),
            AppCommand::ConfirmPluginDisable => self.confirm_plugin_disable(),
            AppCommand::StartAddPluginHook { plugin_id } => {
                self.start_add_plugin_hook(plugin_id.clone());
            }
            AppCommand::StartEditPluginHook { plugin_id, hook } => {
                self.start_edit_plugin_hook(plugin_id.clone(), *hook);
            }
            AppCommand::SetPluginHookDraftEnabled(enabled) => {
                self.set_plugin_hook_draft_enabled(*enabled);
            }
            AppCommand::UpdatePluginHookTarget(target) => {
                self.update_plugin_hook_target(target.clone());
            }
            AppCommand::UpdatePluginHookConfigJson(config_json) => {
                self.update_plugin_hook_config_json(config_json.clone());
            }
            AppCommand::ApplyPluginHookEdit => self.apply_plugin_hook_edit(),
            AppCommand::CancelPluginHookEdit => self.cancel_plugin_hook_edit(),
            AppCommand::ResetPluginHookEdit => self.reset_plugin_hook_edit(),
            AppCommand::SearchPluginDiagnostics(filter) => {
                self.search_plugin_diagnostics(filter.clone());
            }
            AppCommand::SelectPluginDiagnostic(id) => self.select_plugin_diagnostic(id.clone()),
            AppCommand::ClearPluginDiagnostics => self.clear_plugin_diagnostics(),
            AppCommand::InvokeConnectionPluginAction { .. }
            | AppCommand::ClosePluginWindow { .. } => {}
            _ => return false,
        }
        true
    }

    pub(super) fn search_plugins(&mut self, filter: String) {
        self.snapshot.plugins.plugin_filter = filter;
    }

    pub(super) fn select_plugin(&mut self, plugin_id: String) {
        if self.plugin_index(&plugin_id).is_some() {
            self.snapshot.plugins.selected_plugin_id = plugin_id;
            self.snapshot.plugins.feedback = None;
        }
    }

    pub(super) fn select_plugin_surface_tab(&mut self, tab: PluginSurfaceTab) {
        self.snapshot.plugins.active_tab = tab;
    }

    pub(crate) fn set_plugin_installed_path(&mut self, plugin_id: &str, installed_path: String) {
        if let Some(index) = self.plugin_index(plugin_id) {
            self.snapshot.plugins.plugins[index].installed_path = installed_path;
        }
    }

    pub(super) fn set_plugin_enabled(&mut self, plugin_id: String, enabled: bool) {
        let Some(index) = self.plugin_index(&plugin_id) else {
            return;
        };
        if self.snapshot.plugins.plugins[index].status == PluginStatus::UnsupportedLegacy {
            let name = self.snapshot.plugins.plugins[index].name.clone();
            self.snapshot.plugins.feedback = Some(PluginFeedback::warning(format!(
                "{name} is a legacy Java plugin and will be reinitialized from Rust manifests."
            )));
            self.push_diagnostic(Diagnostic::warning(format!(
                "{name} was not enabled because PF4J jars are not a compatibility target."
            )));
            return;
        }
        if self.snapshot.plugins.plugins[index].status == PluginStatus::LoadError && enabled {
            let name = self.snapshot.plugins.plugins[index].name.clone();
            self.snapshot.plugins.feedback = Some(PluginFeedback::error(format!(
                "{name} cannot be enabled until the WASM load error is resolved."
            )));
            return;
        }
        if !enabled && self.snapshot.plugins.plugins[index].enabled {
            let active_hooks: Vec<_> = self.snapshot.plugins.plugins[index]
                .hooks
                .iter()
                .filter(|hook| hook.enabled)
                .map(|hook| hook.hook)
                .collect();
            if !active_hooks.is_empty() {
                let plugin = &self.snapshot.plugins.plugins[index];
                self.snapshot.plugins.disable_confirmation = Some(PluginDisableConfirmation {
                    plugin_id: plugin.id.clone(),
                    plugin_name: plugin.name.clone(),
                    active_hooks,
                });
                self.snapshot.plugins.feedback = Some(PluginFeedback::warning(format!(
                    "Confirm disabling {} because active hooks will be turned off.",
                    plugin.name
                )));
                return;
            }
        }

        let plugin = &mut self.snapshot.plugins.plugins[index];
        plugin.enabled = enabled;
        if enabled {
            if plugin.status == PluginStatus::Disabled {
                plugin.status = PluginStatus::Active;
            }
        } else {
            disable_plugin(plugin);
            if self
                .snapshot
                .workbench
                .detail
                .selected_formatter_plugin_id
                .as_deref()
                == Some(plugin_id.as_str())
            {
                self.snapshot.workbench.detail.selected_formatter_plugin_id = None;
            }
            if self
                .snapshot
                .workbench
                .detail
                .selected_transform_plugin_id
                .as_deref()
                == Some(plugin_id.as_str())
            {
                self.snapshot.workbench.detail.selected_transform_plugin_id = None;
            }
        }
        let plugin_name = plugin.name.clone();
        self.set_persisted_plugin_enabled(plugin_id.clone(), enabled);
        self.sync_persisted_plugin_hooks(&plugin_id);
        self.snapshot.plugins.feedback = Some(PluginFeedback::info(format!(
            "{} {} command queued.",
            plugin_name,
            if enabled { "enable" } else { "disable" }
        )));
    }

    pub(super) fn update_plugin_config_value(
        &mut self,
        plugin_id: String,
        key: String,
        value: String,
    ) {
        let Some(index) = self.plugin_index(&plugin_id) else {
            return;
        };
        let plugin = &mut self.snapshot.plugins.plugins[index];
        let Some(field) = plugin
            .config_fields
            .iter_mut()
            .find(|field| field.key == key)
        else {
            return;
        };
        if field.sensitive {
            self.snapshot.plugins.feedback = Some(PluginFeedback::warning(format!(
                "{} is managed by the secret store and was not edited here.",
                field.label
            )));
            return;
        }

        field.value = value;
        set_field_validation(field);
        refresh_plugin_config_status(plugin);
        self.snapshot.plugins.feedback = Some(PluginFeedback::info(format!(
            "{} config draft updated.",
            plugin.name
        )));
    }

    pub(super) fn apply_plugin_config(&mut self, plugin_id: String) {
        let Some(index) = self.plugin_index(&plugin_id) else {
            return;
        };
        let plugin = &mut self.snapshot.plugins.plugins[index];
        let first_error = validate_config_fields(&mut plugin.config_fields);
        if let Some(error) = first_error {
            plugin.status = PluginStatus::NeedsConfig;
            self.snapshot.plugins.feedback = Some(PluginFeedback::error(format!(
                "{} config was not applied: {error}",
                plugin.name
            )));
            return;
        }

        for field in &mut plugin.config_fields {
            if !field.sensitive {
                field.saved_value = field.value.clone();
            }
        }
        refresh_plugin_config_status(plugin);
        self.snapshot.plugins.feedback = Some(PluginFeedback::info(format!(
            "{} config applied.",
            plugin.name
        )));
    }

    pub(super) fn cancel_plugin_config(&mut self, plugin_id: String) {
        let Some(index) = self.plugin_index(&plugin_id) else {
            return;
        };
        let plugin = &mut self.snapshot.plugins.plugins[index];
        restore_config_values(plugin);
        self.snapshot.plugins.feedback = Some(PluginFeedback::info(format!(
            "{} config edits cancelled.",
            plugin.name
        )));
    }

    pub(super) fn reset_plugin_config(&mut self, plugin_id: String) {
        let Some(index) = self.plugin_index(&plugin_id) else {
            return;
        };
        let plugin = &mut self.snapshot.plugins.plugins[index];
        restore_config_values(plugin);
        self.snapshot.plugins.feedback = Some(PluginFeedback::info(format!(
            "{} config reset to saved values.",
            plugin.name
        )));
    }

    pub(super) fn set_plugin_hook_enabled(
        &mut self,
        plugin_id: String,
        hook: PluginHookKind,
        enabled: bool,
    ) {
        let Some(index) = self.plugin_index(&plugin_id) else {
            return;
        };
        let plugin = &mut self.snapshot.plugins.plugins[index];
        if !plugin.enabled {
            self.snapshot.plugins.feedback = Some(PluginFeedback::warning(format!(
                "Enable {} before changing hook assignments.",
                plugin.name
            )));
            return;
        }
        let Some(assignment) = plugin
            .hooks
            .iter_mut()
            .find(|assignment| assignment.hook == hook)
        else {
            return;
        };
        if assignment.status == PluginHookStatus::Denied && enabled {
            self.snapshot.plugins.feedback = Some(PluginFeedback::warning(format!(
                "{} requires a denied capability grant.",
                hook.label()
            )));
            return;
        }

        assignment.enabled = enabled;
        assignment.status = if enabled {
            PluginHookStatus::Ready
        } else {
            PluginHookStatus::Disabled
        };
        let plugin_name = plugin.name.clone();
        self.sync_persisted_plugin_hooks(&plugin_id);
        self.snapshot.plugins.feedback = Some(PluginFeedback::info(format!(
            "{} {} assignment updated.",
            plugin_name,
            hook.label()
        )));
    }

}

