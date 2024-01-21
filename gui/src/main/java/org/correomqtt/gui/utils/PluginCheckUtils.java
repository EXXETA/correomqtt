package org.correomqtt.gui.utils;

import org.correomqtt.core.settings.SettingsProvider;
import org.correomqtt.core.fileprovider.PluginConfigProvider;

import javax.inject.Inject;
import java.util.ResourceBundle;

public class PluginCheckUtils {
    private static ResourceBundle resources;
    private final SettingsProvider settingsProvider;
    private final AlertHelper alertHelper;

    @Inject
    PluginCheckUtils(SettingsProvider settingsProvider,
                     AlertHelper alertHelper) {
        this.settingsProvider = settingsProvider;
        this.alertHelper = alertHelper;
        resources = ResourceBundle.getBundle("org.correomqtt.i18n", settingsProvider.getSettings().getCurrentLocale());
    }

    public void checkMigration() {

        if (PluginConfigProvider.getInstance().migrationRequired()) {
            boolean confirmed = alertHelper.confirm(
                    resources.getString("correoMqttPluginMigrationTitle"),
                    resources.getString("correoMqttPluginMigrationHeader"),
                    resources.getString("correoMqttPluginMigrationContent"),
                    resources.getString("commonNoButton"),
                    resources.getString("commonYesRecommendedButton")
            );

            if (confirmed) {
                PluginConfigProvider.getInstance().migratePluginFolder();
            }
        }
    }
}
