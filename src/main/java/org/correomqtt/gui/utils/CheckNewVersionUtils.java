package org.correomqtt.gui.utils;

import org.correomqtt.business.provider.SettingsProvider;
import org.correomqtt.business.utils.VersionUtils;
import org.correomqtt.gui.helper.AlertHelper;
import org.json.simple.parser.ParseException;

import java.io.IOException;
import java.util.ResourceBundle;

import static org.correomqtt.business.utils.VendorConstants.GITHUB_LATEST;

public class CheckNewVersionUtils {
    private static ResourceBundle resources = ResourceBundle.getBundle("org.correomqtt.i18n", SettingsProvider.getInstance().getSettings().getCurrentLocale());

    private CheckNewVersionUtils() {
        // nothing to do
    }

    public static void checkNewVersion(boolean showHintIfUpToDate) throws IOException, ParseException {

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
                HostServicesHolder.getInstance().getHostServices().showDocument(GITHUB_LATEST);
            }
        } else if (showHintIfUpToDate) {
            AlertHelper.info(
                    resources.getString("versionUpToDateTitle"),
                    resources.getString("versionUpToDateContent")
            );
        }
    }
}
