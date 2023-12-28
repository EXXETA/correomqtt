package org.correomqtt.gui.controller;

import org.correomqtt.business.dispatcher.ConfigDispatcher;
import org.correomqtt.business.dispatcher.ConfigObserver;
import org.correomqtt.business.dispatcher.PersistPublishHistoryDispatcher;
import org.correomqtt.business.dispatcher.PersistPublishHistoryObserver;
import org.correomqtt.business.dispatcher.PersistSubscriptionHistoryDispatcher;
import org.correomqtt.business.dispatcher.PersistSubscriptionHistoryObserver;
import org.correomqtt.business.dispatcher.SecretStoreDispatcher;
import org.correomqtt.business.dispatcher.SecretStoreObserver;
import org.correomqtt.business.provider.SettingsProvider;
import org.correomqtt.gui.helper.AlertHelper;
import org.correomqtt.gui.utils.PlatformUtils;

import java.util.ResourceBundle;

public class AlertController extends BaseControllerImpl implements
        ConfigObserver,
        SecretStoreObserver,
        PersistPublishHistoryObserver,
        PersistSubscriptionHistoryObserver {

    public static final String ALERT_CONTROLLER_WARN_TITLE = "alertControllerWarnTitle";
    public static final String ALERT_EXCEPTION_TITLE = "Exception";

    private final ResourceBundle resources = ResourceBundle.getBundle("org.correomqtt.i18n", SettingsProvider.getInstance().getSettings().getCurrentLocale());
    private static AlertController instance;

    private AlertController() {
        ConfigDispatcher.getInstance().addObserver(this);
        SecretStoreDispatcher.getInstance().addObserver(this);
        PersistPublishHistoryDispatcher.getInstance().addObserver(this);
        PersistSubscriptionHistoryDispatcher.getInstance().addObserver(this);
    }

    public static void activate() {
        if (instance == null) {
            instance = new AlertController();
        }
    }

    @Override
    public void onConfigDirectoryEmpty() {
        PlatformUtils.runLaterIfNotInFxThread(() -> AlertHelper.warn(resources.getString(ALERT_CONTROLLER_WARN_TITLE),
                resources.getString("alertControllerOnConfigDirectoryEmptyContent")
        ));
    }

    @Override
    public void onConfigDirectoryNotAccessible() {
        PlatformUtils.runLaterIfNotInFxThread(() -> AlertHelper.warn(resources.getString(ALERT_CONTROLLER_WARN_TITLE),
                resources.getString("alertControllerOnConfigDirectoryNotAccessibleContent")
        ));
    }

    @Override
    public void onAppDataNull() {
        PlatformUtils.runLaterIfNotInFxThread(() -> AlertHelper.warn(resources.getString(ALERT_CONTROLLER_WARN_TITLE),
                resources.getString("alertControllerOnAppDataNullContent")
        ));
    }

    @Override
    public void onUserHomeNull() {
        PlatformUtils.runLaterIfNotInFxThread(() -> AlertHelper.warn(resources.getString(ALERT_CONTROLLER_WARN_TITLE),
                resources.getString("alertControllerOnUserHomeNullContent")
        ));
    }

    @Override
    public void onFileAlreadyExists() {
        PlatformUtils.runLaterIfNotInFxThread(() -> AlertHelper.warn(resources.getString("alertControllerOnFileAlreadyExistsTitle"),
                resources.getString("alertControllerOnFileAlreadyExistsContent")
        ));
    }

    @Override
    public void onInvalidPath() {
        PlatformUtils.runLaterIfNotInFxThread(() -> AlertHelper.warn(resources.getString("alertControllerOnInvalidPathTitle"),
                resources.getString("alertControllerOnInvalidPathContent")
        ));
    }

    @Override
    public void onInvalidJsonFormat() {
        PlatformUtils.runLaterIfNotInFxThread(() -> AlertHelper.warn(resources.getString("alertControllerOnInvalidJsonFormatTitle"),
                resources.getString("alertControllerOnInvalidJsonFormatContent")
        ));
    }

    @Override
    public void onSavingFailed() {
        PlatformUtils.runLaterIfNotInFxThread(() -> AlertHelper.warn(resources.getString("alertControllerOnSavingFailedTitle"),
                resources.getString("alertControllerOnSavingFailedContent")
        ));
    }

    @Override
    public void onConnectionsUpdated() {
        // nothing to do
    }

    @Override
    public void onConfigPrepareFailed() {
        PlatformUtils.runLaterIfNotInFxThread(() -> AlertHelper.warn(ALERT_EXCEPTION_TITLE,
                resources.getString("alertControllerOnConfigPrepareFailedContent")
        ));
    }

    @Override
    public void onSettingsUpdated(boolean showRestartRequiredDialog) {
        if (showRestartRequiredDialog) {
            PlatformUtils.runLaterIfNotInFxThread(() -> AlertHelper.info(resources.getString("alertControllerOnSettingsUpdatedTitle"),
                    resources.getString("alertControllerOnSettingsUpdatedContent")
            ));
        }
    }

    @Override
    public void updateSubscriptions(String connectionId) {
        // nothing to do
    }

    @Override
    public void errorReadingSubscriptionHistory(Throwable exception) {
        PlatformUtils.runLaterIfNotInFxThread(() -> AlertHelper.warn(ALERT_EXCEPTION_TITLE,
                resources.getString("alertControllerErrorReadingSubscriptionHistoryContent") + exception.getLocalizedMessage()
        ));
    }

    @Override
    public String getConnectionId() {
        return null;
    }

    @Override
    public void errorReadingPublishHistory(Throwable exception) {
        PlatformUtils.runLaterIfNotInFxThread(() -> AlertHelper.warn(ALERT_EXCEPTION_TITLE,
                resources.getString("alertControllerErrorReadingPublishHistoryContent") + exception.getLocalizedMessage()
        ));
    }

    @Override
    public void errorWritingPublishHistory(Throwable exception) {
        PlatformUtils.runLaterIfNotInFxThread(() -> AlertHelper.warn(ALERT_EXCEPTION_TITLE,
                resources.getString("alertControllerErrorWritingPublishHistoryContent") + exception.getLocalizedMessage()
        ));
    }

    @Override
    public void updatedPublishes(String connectionId) {
        // nothing to do
    }

    @Override
    public void errorWritingSubscriptionHistory(Throwable exception) {
        PlatformUtils.runLaterIfNotInFxThread(() -> AlertHelper.warn(ALERT_EXCEPTION_TITLE,
                resources.getString("alertControllerErrorWritingSubscriptionHistoryContent") + exception.getLocalizedMessage()
        ));
    }

    @Override
    public void onPasswordFileUnreadable() {
        PlatformUtils.runLaterIfNotInFxThread(() -> AlertHelper.warn(
                resources.getString("onPasswordFileUnreadableFailedTitle"),
                resources.getString("onPasswordFileUnreadableFailedContent"),
                true
        ));
    }
}