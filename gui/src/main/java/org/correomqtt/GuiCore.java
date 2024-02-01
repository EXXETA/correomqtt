package org.correomqtt;

import javafx.application.HostServices;
import lombok.Getter;
import lombok.Setter;
import org.correomqtt.core.CoreManager;
import org.correomqtt.core.eventbus.EventBus;
import org.correomqtt.core.fileprovider.HistoryManager;
import org.correomqtt.core.plugin.PluginManager;
import org.correomqtt.core.settings.SettingsManager;
import org.correomqtt.core.utils.ConnectionManager;
import org.correomqtt.gui.keyring.KeyringManager;
import org.correomqtt.gui.model.AppHostServices;
import org.correomqtt.gui.theme.ThemeManager;

import javax.inject.Inject;
import javax.inject.Singleton;

@Getter
@Singleton
public class GuiCore {

    @Getter
    @Setter
    private static MainComponent mainComponent;
    private final ConnectionManager connectionManager;
    private final SettingsManager settingsManager;
    private final HistoryManager historyManager;
    private final PluginManager pluginManager;
    private final KeyringManager keyringManager;
    private final EventBus eventBus;
    private final ThemeManager themeManager;
    private final HostServices hostServices;

    @Inject
    public GuiCore(CoreManager coreManager,
                   KeyringManager keyringManager,
                   EventBus eventBus,
                   ThemeManager themeManager,
                   @AppHostServices HostServices hostServices) {
        this.connectionManager = coreManager.getConnectionManager();
        this.settingsManager = coreManager.getSettingsManager();
        this.historyManager = coreManager.getHistoryManager();
        this.pluginManager = coreManager.getPluginManager();
        this.keyringManager = keyringManager;
        this.eventBus = eventBus;
        this.themeManager = themeManager;
        this.hostServices = hostServices;
    }
}
