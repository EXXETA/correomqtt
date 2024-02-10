package org.correomqtt.gui.utils;

import org.correomqtt.core.fileprovider.PluginConfigProvider;
import org.correomqtt.core.settings.SettingsManager;

import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import java.util.ResourceBundle;

@DefaultBean
public class PluginCheckUtils {
    private final PluginConfigProvider pluginConfigProvider;
    private final AlertHelper alertHelper;
    private ResourceBundle resources;

    @Inject
    public PluginCheckUtils(SettingsManager settingsManager,
                     AlertHelper alertHelper,
                     PluginConfigProvider pluginConfigProvider) {
        this.alertHelper = alertHelper;
        resources = ResourceBundle.getBundle("org.correomqtt.i18n", settingsManager.getSettings().getCurrentLocale());
        this.pluginConfigProvider = pluginConfigProvider;
    }

    public void checkMigration() {

        if (pluginConfigProvider.migrationRequired()) {
            boolean confirmed = alertHelper.confirm(
                    resources.getString("correoMqttPluginMigrationTitle"),
                    resources.getString("correoMqttPluginMigrationHeader"),
                    resources.getString("correoMqttPluginMigrationContent"),
                    resources.getString("commonNoButton"),
                    resources.getString("commonYesRecommendedButton")
            );

            if (confirmed) {
                pluginConfigProvider.migratePluginFolder();
            }
        }
    }
}
