package org.correomqtt.gui.utils;

import org.correomqtt.business.services.ConfigService;
import org.correomqtt.business.utils.VersionUtils;
import org.correomqtt.gui.helper.AlertHelper;
import javafx.util.Pair;
import org.json.simple.parser.ParseException;

import java.io.IOException;
import java.util.ResourceBundle;

public class CheckNewVersionUtils {
    private static ResourceBundle resources = ResourceBundle.getBundle("org.correomqtt.i18n", ConfigService.getInstance().getSettings().getCurrentLocale());

    private CheckNewVersionUtils() {
        // nothing to do
    }

    public static void checkNewVersion(boolean showHintIfUpToDate) throws IOException, ParseException {
        Pair pair = VersionUtils.isNewVersionAvailable();
        if (Boolean.TRUE.equals(pair.getKey())) {
            boolean confirmed = AlertHelper.confirm(
                    resources.getString("correoMqttNewVersionTitle"),
                    pair.getValue() + " " + resources.getString("correoMqttNewVersionHeader"),
                    resources.getString("correoMqttNewVersionContent"),
                    resources.getString("commonNoButton"),
                    resources.getString("commonYesButton")
            );

            if (confirmed) {
                HostServicesHolder.getInstance().getHostServices().showDocument("https://github.com/EXXETA/correomqtt/releases/latest");
            }
        } else if (showHintIfUpToDate) {
            AlertHelper.info(
                    resources.getString("versionUpToDateTitle"),
                    resources.getString("versionUpToDateContent")
            );
        }
    }
}
