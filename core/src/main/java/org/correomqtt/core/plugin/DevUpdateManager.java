package org.correomqtt.core.plugin;

import org.pf4j.PluginManager;
import org.pf4j.PluginWrapper;
import org.pf4j.update.PluginInfo;
import org.pf4j.update.UpdateManager;
import org.pf4j.update.UpdateRepository;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.ArrayList;
import java.util.List;

/**
 * Special UpdateManager for development mode that treats all installed plugins
 * as updateable even if they have the same version.
 */
public class DevUpdateManager extends UpdateManager {

    private static final Logger LOGGER = LoggerFactory.getLogger(DevUpdateManager.class);

    private final PluginManager pluginManager;

    public DevUpdateManager(PluginManager pluginManager, List<UpdateRepository> repositories) {
        super(pluginManager, repositories);
        this.pluginManager = pluginManager;
    }

    /**
     * In dev mode, all plugins are considered to have updates, regardless of their version
     */
    @Override
    public List<PluginInfo> getUpdates() {
        List<PluginInfo> updates = new ArrayList<>();
        List<PluginWrapper> installedPlugins = pluginManager.getPlugins();

        for ( PluginWrapper plugin : installedPlugins ) {
            String pluginId = plugin.getPluginId();
            LOGGER.debug("Check for updates for plugin '{}'", pluginId);

            PluginInfo.PluginRelease lastRelease = getLastPluginRelease(pluginId);
            if (lastRelease == null) {
                continue;
            }

            // In DEV mode, always treat plugins as updateable
            LOGGER.debug("DEV MODE: Treating plugin '{}' as updateable regardless of version", pluginId);
            PluginInfo pluginInfo = getPluginsMap().get(pluginId);
            if (pluginInfo != null) {
                updates.add(pluginInfo);
            }
        }

        return updates;
    }
}
