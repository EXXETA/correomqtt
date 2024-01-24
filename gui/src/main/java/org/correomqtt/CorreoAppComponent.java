package org.correomqtt;

import dagger.BindsInstance;
import dagger.Component;
import javafx.application.HostServices;
import org.correomqtt.core.CoreManager;
import org.correomqtt.core.fileprovider.SecretStoreProvider;
import org.correomqtt.core.plugin.PluginManager;
import org.correomqtt.core.settings.SettingsManager;
import org.correomqtt.core.utils.ConnectionManager;
import org.correomqtt.gui.model.AppHostServices;
import org.correomqtt.gui.theme.ThemeManager;

import javax.inject.Singleton;

@Singleton
@Component
public interface CorreoAppComponent {

    CorreoApp app();

    ThemeManager themeManager();

    CoreManager coreManager();

    PluginManager pluginManager();

    ConnectionManager connectionManager();

    SettingsManager settingsManager();

    SecretStoreProvider secretStoreProvider();

    @Component.Builder
    interface Builder {
        @BindsInstance
        Builder hostServices(@AppHostServices HostServices hostServices);

        CorreoAppComponent build();
    }
}