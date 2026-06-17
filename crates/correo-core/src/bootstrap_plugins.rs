use std::collections::BTreeMap;

use correo_storage::current::PluginStateSettings;

use crate::{
    marketplace_rows_from_repository_json, PluginFeedback, PluginLoadState, PluginSource,
    PluginStatus, PluginSurfaceSnapshot, PluginSurfaceTab,
};

pub(super) fn plugin_surface(
    install_bundled_plugins: bool,
    repository_jsons: &[String],
    bundled_plugin_ids: &[String],
    installed_plugin_ids: &[String],
    installed_plugin_paths: &[(String, String)],
    plugin_states: &BTreeMap<String, PluginStateSettings>,
) -> PluginSurfaceSnapshot {
    let mut marketplace_plugins = Vec::new();
    let mut feedback = None;
    for repository_json in repository_jsons {
        match marketplace_rows_from_repository_json(repository_json) {
            Ok(rows) => merge_marketplace_plugins(&mut marketplace_plugins, rows),
            Err(error) => {
                feedback = Some(PluginFeedback::warning(format!(
                    "A plugin repository was ignored: {error}"
                )));
            }
        }
    }

    let mut plugins = Vec::new();
    if install_bundled_plugins || !installed_plugin_ids.is_empty() {
        for marketplace_plugin in &mut marketplace_plugins {
            let should_install = (install_bundled_plugins
                && bundled_plugin_ids
                    .iter()
                    .any(|id| id == &marketplace_plugin.id))
                || installed_plugin_ids
                    .iter()
                    .any(|id| id == &marketplace_plugin.id);
            if should_install {
                marketplace_plugin.installed_plugin_id = Some(marketplace_plugin.id.clone());
                let mut plugin = marketplace_plugin.to_installed_plugin();
                if bundled_plugin_ids.iter().any(|id| id == &plugin.id) {
                    plugin.source = PluginSource::Bundled;
                }
                if let Some(state) = plugin_states.get(&plugin.id) {
                    plugin.enabled = state.enabled;
                    if !state.enabled && plugin.status == PluginStatus::Active {
                        plugin.status = PluginStatus::Disabled;
                        for hook in &mut plugin.hooks {
                            hook.enabled = false;
                            hook.status = crate::PluginHookStatus::Disabled;
                        }
                    }
                }
                plugin.installed_path = installed_plugin_paths
                    .iter()
                    .find(|(id, _)| id == &marketplace_plugin.id)
                    .map(|(_, path)| path.clone())
                    .unwrap_or_default();
                plugins.push(plugin);
            }
        }
    }

    let selected_plugin_id = plugins
        .first()
        .map(|plugin| plugin.id.clone())
        .unwrap_or_default();
    let selected_marketplace_plugin_id = marketplace_plugins
        .first()
        .map(|plugin| plugin.id.clone())
        .unwrap_or_default();
    let load_state = if plugins.is_empty() && marketplace_plugins.is_empty() {
        PluginLoadState::Empty
    } else {
        PluginLoadState::Ready
    };

    PluginSurfaceSnapshot {
        active_tab: PluginSurfaceTab::Installed,
        load_state,
        plugins,
        marketplace_plugins,
        selected_plugin_id,
        selected_marketplace_plugin_id,
        feedback,
        ..PluginSurfaceSnapshot::default()
    }
}

fn merge_marketplace_plugins(
    marketplace_plugins: &mut Vec<crate::PluginMarketplaceRow>,
    rows: Vec<crate::PluginMarketplaceRow>,
) {
    for row in rows {
        if !marketplace_plugins.iter().any(|plugin| plugin.id == row.id) {
            marketplace_plugins.push(row);
        }
    }
}
