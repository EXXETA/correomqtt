package org.correomqtt.plugin.update;

import org.correomqtt.business.dispatcher.PreloadingDispatcher;
import org.correomqtt.business.services.SettingsService;
import org.correomqtt.business.utils.VersionUtils;
import org.correomqtt.plugin.exception.CorreoMqttPluginUpdateException;
import org.correomqtt.plugin.manager.PluginSystem;
import org.pf4j.update.DefaultUpdateRepository;
import org.pf4j.update.PluginInfo;
import org.pf4j.update.UpdateManager;
import org.pf4j.update.UpdateRepository;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.net.*;
import java.util.Collections;
import java.util.List;
import java.util.ResourceBundle;

import static org.correomqtt.business.utils.VendorConstants.PLUGIN_REPO_URL;

public class PluginUpdateManager {

    private static final Logger LOGGER = LoggerFactory.getLogger(PluginUpdateManager.class);

    private final PluginSystem pluginSystem;
    private ResourceBundle resources = ResourceBundle.getBundle("org.correomqtt.i18n", SettingsService.getInstance().getSettings().getCurrentLocale());

    public PluginUpdateManager(PluginSystem pluginSystem) {
        this.pluginSystem = pluginSystem;
    }

    public void updateSystem() throws Exception {

        LOGGER.info("Start Plugin Update");

        URL versionRepo = new URL(PLUGIN_REPO_URL);
        HttpURLConnection connection = (HttpURLConnection) versionRepo.openConnection();

        if (connection.getResponseCode() == 200) {
            List<UpdateRepository> repos = Collections.singletonList(new DefaultUpdateRepository("bundled", versionRepo, "plugins-" + VersionUtils.getVersion() + ".json"));
            UpdateManager updateManager = new UpdateManager(pluginSystem, repos);
            updateExisitingPlugins(updateManager);
            installNewPlugins(updateManager);
        }else{
            LOGGER.info("No default plugins available.");
        }
    }

    private void installNewPlugins(UpdateManager updateManager) {
        // check for available (new) plugins
        if (updateManager.hasAvailablePlugins()) {
            List<PluginInfo> availablePlugins = updateManager.getAvailablePlugins();
            LOGGER.info("Found {} available plugins", availablePlugins.size());
            for ( PluginInfo plugin : availablePlugins ) {
                LOGGER.info("Found available plugin '{}'", plugin.id);
                PluginInfo.PluginRelease lastRelease = updateManager.getLastPluginRelease(plugin.id);
                String lastVersion = lastRelease.version;
                LOGGER.info("Install plugin '{}' with version {}", plugin.id, lastVersion);
                boolean installed = updateManager.installPlugin(plugin.id, lastVersion);
                if (installed) {
                    LOGGER.info("Installed plugin '{}'", plugin.id);
                    PreloadingDispatcher.getInstance().onProgress(resources.getString("pluginUpdateManagerInstalled") + " " + plugin.id);
                } else {
                    LOGGER.error("Cannot install plugin '{}'", plugin.id);
                }
            }
        } else {
            LOGGER.info("No available plugins found");
        }
    }

    private void updateExisitingPlugins(UpdateManager updateManager) throws Exception {
        // check for updates
        if (updateManager.hasUpdates()) {
            List<PluginInfo> updates = updateManager.getUpdates();
            LOGGER.info("Found {} plugin updates", updates.size());
            for ( PluginInfo plugin : updates ) {
                LOGGER.info("Found update for plugin '{}'", plugin.id);
                PluginInfo.PluginRelease lastRelease = updateManager.getLastPluginRelease(plugin.id);
                String lastVersion = lastRelease.version;
                String installedVersion = pluginSystem.getPlugin(plugin.id).getDescriptor().getVersion();
                LOGGER.info("Update plugin '{}' from version {} to version {}", plugin.id, installedVersion, lastVersion);
                boolean updated;
                try {
                    updated = updateManager.updatePlugin(plugin.id, lastVersion);
                } catch (Throwable t) {
                    updated = false;
                }
                if (updated) {
                    LOGGER.info("Updated plugin '{}'", plugin.id);
                    PreloadingDispatcher.getInstance().onProgress(resources.getString("pluginUpdateManagerUpdated") + " " + plugin.id);
                } else {
                    LOGGER.error("Cannot update plugin '{}'", plugin.id);
                    throw new CorreoMqttPluginUpdateException(plugin.id);
                }
            }
        } else {
            LOGGER.info("No updates found");
        }
    }
}
