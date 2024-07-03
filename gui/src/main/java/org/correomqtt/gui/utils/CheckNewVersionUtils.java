package org.correomqtt.gui.utils;

import javafx.application.HostServices;
import org.correomqtt.HostServicesWrapper;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import org.correomqtt.core.exception.CorreoMqttUnableToCheckVersionException;
import org.correomqtt.core.settings.SettingsManager;
import org.correomqtt.core.utils.VendorConstants;
import org.correomqtt.core.utils.VersionUtils;

import java.io.IOException;
import java.util.ResourceBundle;

@DefaultBean
public class CheckNewVersionUtils {
    private final ResourceBundle resources;
    private final HostServices hostServices;
    private final AlertHelper alertHelper;

    @Inject
    public CheckNewVersionUtils(SettingsManager settingsManager,
                         AlertHelper alertHelper,
                         HostServicesWrapper hostServices) {
        this.alertHelper = alertHelper;
        resources = ResourceBundle.getBundle("org.correomqtt.i18n", settingsManager.getSettings().getCurrentLocale());

        this.hostServices = hostServices.getHostServices();
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
                hostServices.showDocument(VendorConstants.getGithubLatest());

            }
        } else if (showHintIfUpToDate) {
            alertHelper.info(
                    resources.getString("versionUpToDateTitle"),
                    resources.getString("versionUpToDateContent")
            );
        }
    }
}
