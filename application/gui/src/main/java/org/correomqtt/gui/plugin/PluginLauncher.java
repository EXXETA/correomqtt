package org.correomqtt.gui.plugin;

import javafx.application.Preloader;
import org.correomqtt.core.plugin.PluginManager;
import org.correomqtt.core.plugin.repository.BundledPluginList;
import org.correomqtt.core.settings.SettingsManager;
import org.correomqtt.di.Inject;
import org.correomqtt.di.SingletonBean;
import org.correomqtt.di.SoyDi;
import org.correomqtt.preloader.PreloaderNotification;
import org.pf4j.PluginState;
import org.pf4j.update.PluginInfo;
import org.pf4j.update.UpdateManager;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.ResourceBundle;
import java.util.function.Consumer;

@SingletonBean
public class PluginLauncher {

    private static final Logger LOGGER = LoggerFactory.getLogger(PluginLauncher.class);

    private final ResourceBundle resources;

    private final PluginManager pluginManager;

    private Consumer<Preloader.PreloaderNotification> notifyPreloader;

    @Inject
    public PluginLauncher(PluginManager pluginManager, SettingsManager settingsManager) {
        this.pluginManager = pluginManager;
        resources = ResourceBundle.getBundle("org.correomqtt.i18n", settingsManager.getSettings().getCurrentLocale());
    }

    public void start(boolean doPluginUpdates) {
        try {
            notifyPreloader.accept(new PreloaderNotification(resources.getString("preloaderLoadPlugins")));
            pluginManager.loadPlugins();
            if (doPluginUpdates) {
                notifyPreloader.accept(new PreloaderNotification(resources.getString("preloaderUpdatePlugins")));
                updateSystem();
            }
            notifyPreloader.accept(new PreloaderNotification(resources.getString("preloaderStartPlugins")));
            pluginManager.startPlugins();
            pluginManager.getPlugins()
                    .stream()
                    .filter(pw -> pw.getPluginState() == PluginState.STARTED)
                    .forEach(pw -> {
                                SoyDi.addClassLoader(pw.getPluginClassLoader());
                                SoyDi.scan(pw.getPlugin().getClass().getPackageName());
                            }
                    );
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

    private int installBundledPlugins(UpdateManager updateManager,
                                      PluginManager pluginManager,
                                      BundledPluginList.BundledPlugins bundledPlugins) {
        int installedPlugins = 0;
        if(bundledPlugins == null){
            return installedPlugins;
        }
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
            notifyPreloader.accept(new PreloaderNotification(resources.getString("pluginUpdateManagerInstalling") + " " + pluginId));
            String lastVersion = lastRelease.version;
            try {
                boolean installed = updateManager.installPlugin(pluginId, lastVersion);
                if (installed) {
                    LOGGER.info("Installed bundled plugin '{}@{}'", pluginId, lastVersion);
                    notifyPreloader.accept(new PreloaderNotification(resources.getString("pluginUpdateManagerInstalling")));
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
                notifyPreloader.accept(new PreloaderNotification(resources.getString("pluginUpdateManagerUninstalled") + " " + pluginId));
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
            notifyPreloader.accept(new PreloaderNotification(resources.getString("pluginUpdateManagerUpdating") + " " + plugin.id));
            PluginInfo.PluginRelease lastRelease = updateManager.getLastPluginRelease(plugin.id);
            String lastVersion = lastRelease.version;
            String installedVersion = pluginManager.getPlugin(plugin.id).getDescriptor().getVersion();
            try {
                if (updateManager.updatePlugin(plugin.id, lastVersion)) {
                    LOGGER.info("Updated plugin '{}@{}' to '{}@{}'", plugin.id, installedVersion, plugin.id, lastVersion);
                    updatedPlugins++;
                    notifyPreloader.accept(new PreloaderNotification(resources.getString("pluginUpdateManagerUpdated") + " " + plugin.id));
                } else {
                    LOGGER.warn("Cannot update plugin '{}'", plugin.id);
                }
            } catch (Exception e) {
                LOGGER.error("Plugin update failed: '{}'", plugin.id, e);
            }
        }
        return updatedPlugins;
    }

    public void onNotifyPreloader(Consumer<Preloader.PreloaderNotification> notifyPreloader) {
        this.notifyPreloader = notifyPreloader;
    }
}
