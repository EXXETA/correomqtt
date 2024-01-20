package org.correomqtt.gui.utils;

import org.correomqtt.core.exception.CorreoMqttUnableToCheckVersionException;
import org.correomqtt.business.settings.SettingsProvider;
import org.correomqtt.core.utils.VendorConstants;
import org.correomqtt.core.utils.VersionUtils;

import java.io.IOException;
import java.util.ResourceBundle;

public class CheckNewVersionUtils {
    private static ResourceBundle resources = ResourceBundle.getBundle("org.correomqtt.i18n", SettingsProvider.getInstance().getSettings().getCurrentLocale());

    private CheckNewVersionUtils() {
        // nothing to do
    }

    public static void checkNewVersion(boolean showHintIfUpToDate) throws IOException, CorreoMqttUnableToCheckVersionException {

        String newVersion = VersionUtils.isNewerVersionAvailable();
        if (newVersion != null) {
            boolean confirmed = AlertHelper.confirm(
                    resources.getString("correoMqttNewVersionTitle"),
                    newVersion + " " + resources.getString("correoMqttNewVersionHeader"),
                    resources.getString("correoMqttNewVersionContent"),
                    resources.getString("commonNoButton"),
                    resources.getString("commonYesButton")
            );

            if (confirmed) {
                HostServicesHolder.getInstance().getHostServices().showDocument(VendorConstants.GITHUB_LATEST());
            }
        } else if (showHintIfUpToDate) {
            AlertHelper.info(
                    resources.getString("versionUpToDateTitle"),
                    resources.getString("versionUpToDateContent")
            );
        }
    }
}
