package org.correomqtt.plugin;

import org.apache.commons.io.FileUtils;
import org.correomqtt.business.dispatcher.PreloadingDispatcher;
import org.correomqtt.business.dispatcher.StartupDispatcher;
import org.correomqtt.business.services.BaseUserFileService;
import org.correomqtt.business.services.ConfigService;
import org.correomqtt.business.utils.VersionUtils;
import org.correomqtt.plugin.manager.PluginManager;
import org.pf4j.update.DefaultUpdateRepository;
import org.pf4j.update.PluginInfo;
import org.pf4j.update.UpdateManager;
import org.pf4j.update.UpdateRepository;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.File;
import java.io.IOException;
import java.net.HttpURLConnection;
import java.net.URL;
import java.net.UnknownHostException;
import java.nio.charset.StandardCharsets;
import java.text.SimpleDateFormat;
import java.util.Collections;
import java.util.Date;
import java.util.List;
import java.util.ResourceBundle;

import static org.correomqtt.business.utils.VendorConstants.PLUGIN_REPO_URL;

public class PluginSystem {

    private static final Logger LOGGER = LoggerFactory.getLogger(PluginSystem.class);

    private ResourceBundle resources = ResourceBundle.getBundle("org.correomqtt.i18n", ConfigService.getInstance().getSettings().getCurrentLocale());
    private String pluginsDisabledPath = new BaseUserFileService().getTargetDirectoryPath() + File.separator + "plugins.disabled.";

    public void start() throws IOException {
        start(true);
    }

    public void start(boolean retry) throws IOException {

        String pluginDir = new BaseUserFileService().getTargetDirectoryPath() + File.separator + "plugins";
        String brokenFile = pluginDir + File.separator + "broken";

        if(new File(brokenFile).exists()){
            disableAllPluginsDueToErrors();
        }

        PluginManager pluginManager = PluginManager.getInstance();

        try {
            PreloadingDispatcher.getInstance().onProgress(resources.getString("preloaderLoadPlugins"));
            pluginManager.loadPlugins();
            PreloadingDispatcher.getInstance().onProgress(resources.getString("preloaderUpdatePlugins"));
            updateSystem(pluginManager);
            PreloadingDispatcher.getInstance().onProgress(resources.getString("preloaderStartPlugins"));
            pluginManager.startPlugins();
        } catch (UnknownHostException ue) {
            LOGGER.info("No internet connection for updating plugins");
        } catch (Exception | NoClassDefFoundError e) {
            LOGGER.error("Error or Exception during loading plugins ", e);
            if(new BaseUserFileService().isWindows()) {
                // on windows it is not possible to move plugin files after plugin system start: https://github.com/pf4j/pf4j/pull/356
                FileUtils.writeStringToFile(new File(brokenFile), e.getMessage(), StandardCharsets.UTF_8);
                StartupDispatcher.getInstance().onPluginLoadFailed(); // This will exit the application after dialog
            }else{
                if(retry) {
                    FileUtils.writeStringToFile(new File(brokenFile), e.getMessage(), StandardCharsets.UTF_8);
                    disableAllPluginsDueToErrors();
                    PluginManager.resetInstance();
                    start(false);
                }else {
                    StartupDispatcher.getInstance().onPluginLoadFailed();  // This will exit the application after dialog
                }
            }
        }
    }

    private void disableAllPluginsDueToErrors() throws IOException {
        final String disabledPath = pluginsDisabledPath + new SimpleDateFormat("yyyy-dd-MM_HH-mm-ss").format(new Date());
        FileUtils.moveDirectory(new File(new BaseUserFileService().getTargetDirectoryPath() + File.separator + "plugins"),
                                new File(disabledPath));
        StartupDispatcher.getInstance().onPluginUpdateFailed(disabledPath);
    }

    private void updateSystem(PluginManager pluginManager) throws IOException {

        LOGGER.info("Start Plugin Update");

        URL versionRepo = new URL(PLUGIN_REPO_URL);
        HttpURLConnection connection = (HttpURLConnection) versionRepo.openConnection();

        if (connection.getResponseCode() == 200) {
            List<UpdateRepository> repos = Collections.singletonList(new DefaultUpdateRepository("bundled", versionRepo, "plugins-" + VersionUtils.getVersion() + ".json"));
            UpdateManager updateManager = new UpdateManager(pluginManager, repos);
            updateExisitingPlugins(updateManager, pluginManager);
            installNewPlugins(updateManager);
        } else {
            LOGGER.info("No default plugins available.");
        }
    }

    private void installNewPlugins(UpdateManager updateManager) {
        // check for available (new) plugins
        if (updateManager.hasAvailablePlugins()) {
            List<PluginInfo> availablePlugins = updateManager.getAvailablePlugins();
            LOGGER.info("Found {} available plugins", availablePlugins.size());
            for ( PluginInfo plugin : availablePlugins ) {
                PreloadingDispatcher.getInstance().onProgress(resources.getString("pluginUpdateManagerInstalling") + " " + plugin.id);
                LOGGER.info("Found available plugin '{}'", plugin.id);
                PluginInfo.PluginRelease lastRelease = updateManager.getLastPluginRelease(plugin.id);
                String lastVersion = lastRelease.version;
                LOGGER.info("Install plugin '{}' with version {}", plugin.id, lastVersion);
                try {
                    boolean installed = updateManager.installPlugin(plugin.id, lastVersion);
                    if (installed) {
                        LOGGER.info("Installed plugin '{}'", plugin.id);
                        PreloadingDispatcher.getInstance().onProgress(resources.getString("pluginUpdateManagerInstalled") + " " + plugin.id);
                    } else {
                        LOGGER.error("Cannot install plugin '{}'", plugin.id);
                    }
                } catch (Exception e) {
                    LOGGER.error("Plugin installation failed: '{}'", plugin.id, e);
                }
            }
        } else {
            LOGGER.info("No available plugins found");
        }
    }

    private void updateExisitingPlugins(UpdateManager updateManager, PluginManager pluginManager) {
        // check for updates
        if (updateManager.hasUpdates()) {
            List<PluginInfo> updates = updateManager.getUpdates();
            LOGGER.info("Found {} plugin updates", updates.size());
            for ( PluginInfo plugin : updates ) {
                PreloadingDispatcher.getInstance().onProgress(resources.getString("pluginUpdateManagerUpdating") + " " + plugin.id);
                LOGGER.info("Found update for plugin '{}'", plugin.id);
                PluginInfo.PluginRelease lastRelease = updateManager.getLastPluginRelease(plugin.id);
                String lastVersion = lastRelease.version;
                String installedVersion = pluginManager.getPlugin(plugin.id).getDescriptor().getVersion();
                LOGGER.info("Update plugin '{}' from version {} to version {}", plugin.id, installedVersion, lastVersion);
                try {
                    if (updateManager.updatePlugin(plugin.id, lastVersion)) {
                        LOGGER.info("Updated plugin '{}'", plugin.id);
                        PreloadingDispatcher.getInstance().onProgress(resources.getString("pluginUpdateManagerUpdated") + " " + plugin.id);
                    } else {
                        LOGGER.warn("Cannot update plugin '{}'", plugin.id);
                    }
                } catch (Exception e) {
                    LOGGER.error("Plugin update failed: '{}'", plugin.id, e);
                }
            }
        } else {
            LOGGER.info("No updates found");
        }
    }
}
