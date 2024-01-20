package org.correomqtt;

import org.correomqtt.business.settings.SettingsProvider;
import org.correomqtt.core.eventbus.EventBus;
import org.correomqtt.core.plugin.PluginManager;
import org.correomqtt.core.plugin.repository.BundledPluginList;
import org.pf4j.update.PluginInfo;
import org.pf4j.update.UpdateManager;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import javax.inject.Inject;
import javax.inject.Singleton;
import java.util.ResourceBundle;

@Singleton
public class PluginLauncher {

    private static final Logger LOGGER = LoggerFactory.getLogger(PluginLauncher.class);

    private final ResourceBundle resources = ResourceBundle.getBundle("org.correomqtt.i18n", SettingsProvider.getInstance().getSettings().getCurrentLocale());
    private final PluginManager pluginManager;

    @Inject
    PluginLauncher(PluginManager pluginManager) {
        this.pluginManager = pluginManager;
    }

    public void start(boolean doPluginUpdates) {

        try {
            EventBus.fireAsync(new PreloadingProgressEvent(resources.getString("preloaderLoadPlugins")));
            pluginManager.loadPlugins();
            if (doPluginUpdates) {
                EventBus.fireAsync(new PreloadingProgressEvent(resources.getString("preloaderUpdatePlugins")));
                updateSystem();
            }
            EventBus.fireAsync(new PreloadingProgressEvent(resources.getString("preloaderStartPlugins")));
            pluginManager.startPlugins();
        } catch (Exception e) {
            LOGGER.error("Error or Exception during loading plugins ", e);
        }
    }

    private void updateSystem() {

        UpdateManager updateManager = pluginManager.getUpdateManager();
        BundledPluginList.BundledPlugins bundledPlugins = pluginManager.getBundledPlugins();

        int updatedPlugins = updateExisitingPlugins(updateManager, pluginManager);
        int installedPlugins = installBundledPlugins(updateManager, pluginManager, bundledPlugins);
        int uninstalledPlugins = uninstallBundledPlugins(pluginManager, bundledPlugins);

        LOGGER.info("Plugin Update: Updated({}), Installed({}), Uninstalled({})", updatedPlugins, installedPlugins, uninstalledPlugins);
    }

    private int installBundledPlugins(UpdateManager updateManager, PluginManager pluginManager, BundledPluginList.BundledPlugins bundledPlugins) {

        int installedPlugins = 0;
        for (String pluginId : bundledPlugins.getInstall()) {

            // Already installed?
            if (pluginManager.getPlugin(pluginId) != null) {
                LOGGER.info("Skip installing bundled plugin '{}', as it is already installed.", pluginId);
                continue;
            }

            PluginInfo.PluginRelease lastRelease = updateManager.getLastPluginRelease(pluginId);

            // Plugin available?
            if (lastRelease == null) {
                LOGGER.warn("Skip installing bundled plugin '{}', as it is not available in repositories.", pluginId);
                continue;
            }

            EventBus.fireAsync(new PreloadingProgressEvent(resources.getString("pluginUpdateManagerInstalling") + " " + pluginId));
            String lastVersion = lastRelease.version;
            try {
                boolean installed = updateManager.installPlugin(pluginId, lastVersion);
                if (installed) {
                    LOGGER.info("Installed bundled plugin '{}@{}'", pluginId, lastVersion);
                    EventBus.fireAsync(new PreloadingProgressEvent(resources.getString("pluginUpdateManagerInstalled") + " " + pluginId));
                    installedPlugins++;
                } else {
                    LOGGER.error("Cannot install bundled plugin '{}'", pluginId);
                }
            } catch (Exception e) {
                LOGGER.error("Plugin installation failed: '{}'", pluginId, e);
            }
        }

        return installedPlugins;
    }

    private int uninstallBundledPlugins(PluginManager pluginManager, BundledPluginList.BundledPlugins bundledPlugins) {
        int uninstalledPlugins = 0;
        for (String pluginId : bundledPlugins.getUninstall()) {
            // Already uninstalled?
            if (pluginManager.getPlugin(pluginId) == null) {
                continue;
            }

            boolean uninstalled = pluginManager.deletePlugin(pluginId);
            if (uninstalled) {
                LOGGER.info("Uninstalled deprecated plugin '{}'", pluginId);
                EventBus.fireAsync(new PreloadingProgressEvent(resources.getString("pluginUpdateManagerUninstalled") + " " + pluginId));
                return 1;
            } else {
                LOGGER.error("Cannot uninstall plugin '{}'", pluginId);
            }
            uninstalledPlugins++;
        }
        return uninstalledPlugins;
    }

    private int updateExisitingPlugins(UpdateManager updateManager, PluginManager pluginManager) {
        // check for updates
        int updatedPlugins = 0;
        for (PluginInfo plugin : updateManager.getUpdates()) {
            EventBus.fireAsync(new PreloadingProgressEvent(resources.getString("pluginUpdateManagerUpdating") + " " + plugin.id));
            PluginInfo.PluginRelease lastRelease = updateManager.getLastPluginRelease(plugin.id);
            String lastVersion = lastRelease.version;
            String installedVersion = pluginManager.getPlugin(plugin.id).getDescriptor().getVersion();
            try {
                if (updateManager.updatePlugin(plugin.id, lastVersion)) {
                    LOGGER.info("Updated plugin '{}@{}' to '{}@{}'", plugin.id, installedVersion, plugin.id, lastVersion);
                    updatedPlugins++;
                    EventBus.fireAsync(new PreloadingProgressEvent(resources.getString("pluginUpdateManagerUpdated") + " " + plugin.id));
                } else {
                    LOGGER.warn("Cannot update plugin '{}'", plugin.id);
                }
            } catch (Exception e) {
                LOGGER.error("Plugin update failed: '{}'", plugin.id, e);
            }
        }
        return updatedPlugins;
    }
}
