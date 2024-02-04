package org.correomqtt.core;

import lombok.Getter;
import org.correomqtt.core.fileprovider.HistoryManager;
import org.correomqtt.core.plugin.PluginManager;
import org.correomqtt.core.settings.SettingsManager;
import org.correomqtt.core.utils.ConnectionManager;

import org.correomqtt.di.Inject;
import org.correomqtt.di.SingletonBean;

@Getter
@SingletonBean
public class CoreManager {

    private final ConnectionManager connectionManager;
    private final SettingsManager settingsManager;
    private final HistoryManager historyManager;
    private final PluginManager pluginManager;

    @Inject
    public CoreManager(ConnectionManager connectionManager,
                       SettingsManager settingsManager,
                       HistoryManager historyManager,
                       PluginManager pluginManager) {
        this.connectionManager = connectionManager;
        this.settingsManager = settingsManager;
        this.historyManager = historyManager;
        this.pluginManager = pluginManager;
    }
}
