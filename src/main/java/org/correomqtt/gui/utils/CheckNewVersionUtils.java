package org.correomqtt.gui.utils;

import org.correomqtt.business.services.ConfigService;
import org.correomqtt.business.utils.VersionUtils;
import org.correomqtt.gui.helper.AlertHelper;
import javafx.util.Pair;
import org.json.simple.parser.ParseException;

import java.io.IOException;
import java.util.ResourceBundle;
import java.util.concurrent.CountDownLatch;

public class CheckNewVersionUtils {
    private static ResourceBundle resources = ResourceBundle.getBundle("org.correomqtt.i18n", ConfigService.getInstance().getSettings().getCurrentLocale());

    private CheckNewVersionUtils() {
        // nothing to do
    }

    public static void checkNewVersion(boolean showHintIfUpToDate, CountDownLatch countDownLatch) throws IOException, ParseException {
        Pair pair = VersionUtils.isNewerVersionAvailable();
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
            if ( countDownLatch!= null ) {
                countDownLatch.countDown();
            }
        } else if (showHintIfUpToDate) {
            AlertHelper.info(
                    resources.getString("versionUpToDateTitle"),
                    resources.getString("versionUpToDateContent")
            );
            countDownLatch.countDown();
        } else {
            countDownLatch.countDown();
        }
    }
}
