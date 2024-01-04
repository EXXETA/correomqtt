package org.correomqtt.gui.utils;

import org.correomqtt.business.fileprovider.PluginConfigProvider;
import org.correomqtt.business.fileprovider.SettingsProvider;

import java.util.ResourceBundle;

public class PluginCheckUtils {
    private static ResourceBundle resources = ResourceBundle.getBundle("org.correomqtt.i18n", SettingsProvider.getInstance().getSettings().getCurrentLocale());

    private PluginCheckUtils() {
        // nothing to do
    }

    public static void checkMigration() {

        if (PluginConfigProvider.getInstance().migrationRequired()) {
            boolean confirmed = AlertHelper.confirm(
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
