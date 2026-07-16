impl AppModel {
    pub(super) fn cancel_plugin_disable(&mut self) {
        let Some(confirmation) = self.snapshot.plugins.disable_confirmation.take() else {
            return;
        };
        self.snapshot.plugins.feedback = Some(PluginFeedback::info(format!(
            "{} remains enabled; disable was cancelled.",
            confirmation.plugin_name
        )));
    }

    pub(super) fn confirm_plugin_disable(&mut self) {
        let Some(confirmation) = self.snapshot.plugins.disable_confirmation.take() else {
            return;
        };
        let Some(index) = self.plugin_index(&confirmation.plugin_id) else {
            return;
        };
        let plugin = &mut self.snapshot.plugins.plugins[index];
        disable_plugin(plugin);
        let plugin_name = plugin.name.clone();
        self.set_persisted_plugin_enabled(confirmation.plugin_id.clone(), false);
        self.sync_persisted_plugin_hooks(&confirmation.plugin_id);
        self.snapshot.plugins.feedback = Some(PluginFeedback::info(format!(
            "{} disabled and active hooks turned off.",
            plugin_name
        )));
    }

    fn set_persisted_plugin_enabled(&mut self, plugin_id: String, enabled: bool) {
        self.snapshot
            .global_settings
            .plugin_states
            .insert(plugin_id, PluginStateSnapshot { enabled });
    }

    fn sync_persisted_plugin_hooks(&mut self, plugin_id: &str) {
        let Some(plugin) = self
            .snapshot
            .plugins
            .plugins
            .iter()
            .find(|plugin| plugin.id == plugin_id)
        else {
            self.snapshot.global_settings.plugin_hooks.remove(plugin_id);
            return;
        };
        let hooks = plugin
            .hooks
            .iter()
            .map(|hook| PluginHookSettingsSnapshot {
                hook: hook.hook,
                enabled: hook.enabled,
                target: hook.target.clone(),
                config_json: hook.config_json.clone(),
            })
            .collect::<Vec<_>>();
        if hooks.is_empty() {
            self.snapshot.global_settings.plugin_hooks.remove(plugin_id);
        } else {
            self.snapshot
                .global_settings
                .plugin_hooks
                .insert(plugin_id.to_owned(), hooks);
        }
    }

    pub(super) fn start_add_plugin_hook(&mut self, plugin_id: String) {
        let Some(index) = self.plugin_index(&plugin_id) else {
            return;
        };
        let plugin = &self.snapshot.plugins.plugins[index];
        let hook = PluginHookKind::ALL
            .into_iter()
            .find(|hook| {
                !plugin
                    .hooks
                    .iter()
                    .any(|assignment| assignment.hook == *hook)
            })
            .unwrap_or(PluginHookKind::IncomingTransform);
        self.snapshot.plugins.active_tab = PluginSurfaceTab::Hooks;
        self.snapshot.plugins.hook_editor = Some(PluginHookEditor {
            plugin_id: plugin.id.clone(),
            plugin_name: plugin.name.clone(),
            original: None,
            draft: PluginHookDraft {
                hook,
                enabled: true,
                target: String::new(),
                config_json: "{}".to_owned(),
            },
            error: None,
        });
    }

    pub(super) fn start_edit_plugin_hook(&mut self, plugin_id: String, hook: PluginHookKind) {
        let Some(index) = self.plugin_index(&plugin_id) else {
            return;
        };
        let plugin = &self.snapshot.plugins.plugins[index];
        let Some(assignment) = plugin
            .hooks
            .iter()
            .find(|assignment| assignment.hook == hook)
        else {
            return;
        };
        let draft = PluginHookDraft::from(assignment);
        self.snapshot.plugins.active_tab = PluginSurfaceTab::Hooks;
        self.snapshot.plugins.hook_editor = Some(PluginHookEditor {
            plugin_id: plugin.id.clone(),
            plugin_name: plugin.name.clone(),
            original: Some(draft.clone()),
            draft,
            error: None,
        });
    }

    pub(super) fn set_plugin_hook_draft_enabled(&mut self, enabled: bool) {
        if let Some(editor) = &mut self.snapshot.plugins.hook_editor {
            editor.draft.enabled = enabled;
            editor.error = None;
        }
    }

    pub(super) fn update_plugin_hook_target(&mut self, target: String) {
        if let Some(editor) = &mut self.snapshot.plugins.hook_editor {
            editor.draft.target = target;
            editor.error = None;
        }
    }

    pub(super) fn update_plugin_hook_config_json(&mut self, config_json: String) {
        if let Some(editor) = &mut self.snapshot.plugins.hook_editor {
            editor.draft.config_json = config_json;
            editor.error = None;
        }
    }

    pub(super) fn apply_plugin_hook_edit(&mut self) {
        let Some(editor) = self.snapshot.plugins.hook_editor.clone() else {
            return;
        };
        if let Err(error) = validate_hook_draft(&editor.draft) {
            if let Some(active_editor) = &mut self.snapshot.plugins.hook_editor {
                active_editor.error = Some(error.clone());
            }
            self.snapshot.plugins.feedback = Some(PluginFeedback::error(format!(
                "{} hook edit was not applied: {error}",
                editor.plugin_name
            )));
            return;
        }

        let Some(index) = self.plugin_index(&editor.plugin_id) else {
            return;
        };
        let plugin = &mut self.snapshot.plugins.plugins[index];
        let assignment = assignment_from_draft(&editor.draft);
        if let Some(existing) = plugin
            .hooks
            .iter_mut()
            .find(|existing| existing.hook == editor.draft.hook)
        {
            *existing = assignment;
        } else {
            plugin.hooks.push(assignment);
        }
        if plugin.enabled && plugin.status == PluginStatus::Disabled {
            plugin.status = PluginStatus::Active;
        }
        let plugin_name = plugin.name.clone();
        self.sync_persisted_plugin_hooks(&editor.plugin_id);
        self.snapshot.plugins.hook_editor = None;
        self.snapshot.plugins.feedback = Some(PluginFeedback::info(format!(
            "{} {} hook applied.",
            plugin_name,
            editor.draft.hook.label()
        )));
    }

    pub(super) fn cancel_plugin_hook_edit(&mut self) {
        let Some(editor) = self.snapshot.plugins.hook_editor.take() else {
            return;
        };
        self.snapshot.plugins.feedback = Some(PluginFeedback::info(format!(
            "{} hook edit cancelled.",
            editor.plugin_name
        )));
    }

    pub(super) fn reset_plugin_hook_edit(&mut self) {
        let Some(editor) = &mut self.snapshot.plugins.hook_editor else {
            return;
        };
        if let Some(original) = &editor.original {
            editor.draft = original.clone();
        } else {
            editor.draft.enabled = true;
            editor.draft.target.clear();
            editor.draft.config_json = "{}".to_owned();
        }
        editor.error = None;
        self.snapshot.plugins.feedback =
            Some(PluginFeedback::info("Hook editor reset to saved values."));
    }

    pub(super) fn search_plugin_diagnostics(&mut self, filter: String) {
        self.snapshot.plugins.diagnostic_filter = filter;
    }

    pub(super) fn select_plugin_diagnostic(&mut self, id: String) {
        if self
            .snapshot
            .plugins
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.id == id)
        {
            self.snapshot.plugins.selected_diagnostic_id = Some(id);
        }
    }

    pub(super) fn clear_plugin_diagnostics(&mut self) {
        for plugin in &mut self.snapshot.plugins.plugins {
            plugin.diagnostics.clear();
            if matches!(plugin.status, PluginStatus::HookFailed) {
                plugin.status = if plugin.enabled {
                    PluginStatus::Active
                } else {
                    PluginStatus::Disabled
                };
            }
        }
        self.snapshot.plugins.selected_diagnostic_id = None;
        self.snapshot.plugins.feedback = Some(PluginFeedback::info("Plugin diagnostics cleared."));
    }

    fn plugin_index(&self, plugin_id: &str) -> Option<usize> {
        self.snapshot
            .plugins
            .plugins
            .iter()
            .position(|plugin| plugin.id == plugin_id)
    }
}
