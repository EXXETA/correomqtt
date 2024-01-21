package org.correomqtt.gui.utils;

import javafx.application.HostServices;
import org.correomqtt.gui.model.AppHostServices;
import org.correomqtt.core.settings.SettingsProvider;
import org.correomqtt.core.exception.CorreoMqttUnableToCheckVersionException;
import org.correomqtt.core.utils.VendorConstants;
import org.correomqtt.core.utils.VersionUtils;

import javax.inject.Inject;
import java.io.IOException;
import java.util.ResourceBundle;

public class CheckNewVersionUtils {
    private final ResourceBundle resources;
    private final HostServices hostServices;
    private final AlertHelper alertHelper;

    @Inject
    CheckNewVersionUtils(SettingsProvider settingsProvider,
                         AlertHelper alertHelper,
                         @AppHostServices HostServices hostServices) {
        this.alertHelper = alertHelper;
        resources = ResourceBundle.getBundle("org.correomqtt.i18n", settingsProvider.getSettings().getCurrentLocale());

        this.hostServices = hostServices;
    }

    public void checkNewVersion(boolean showHintIfUpToDate) throws IOException, CorreoMqttUnableToCheckVersionException {

        String newVersion = VersionUtils.isNewerVersionAvailable();
        if (newVersion != null) {
            boolean confirmed = alertHelper.confirm(
                    resources.getString("correoMqttNewVersionTitle"),
                    newVersion + " " + resources.getString("correoMqttNewVersionHeader"),
                    resources.getString("correoMqttNewVersionContent"),
                    resources.getString("commonNoButton"),
                    resources.getString("commonYesButton")
            );

            if (confirmed) {
                hostServices.showDocument(VendorConstants.GITHUB_LATEST());

            }
        } else if (showHintIfUpToDate) {
            alertHelper.info(
                    resources.getString("versionUpToDateTitle"),
                    resources.getString("versionUpToDateContent")
            );
        }
    }
}
