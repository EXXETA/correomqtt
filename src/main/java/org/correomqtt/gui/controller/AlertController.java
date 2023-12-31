package org.correomqtt.gui.controller;

import org.correomqtt.business.dispatcher.ConfigDispatcher;
import org.correomqtt.business.dispatcher.ConfigObserver;
import org.correomqtt.business.eventbus.EventBus;
import org.correomqtt.business.eventbus.Subscribe;
import org.correomqtt.business.fileprovider.PersistPublishHistoryReadFailedEvent;
import org.correomqtt.business.fileprovider.PersistPublishHistoryWriteFailedEvent;
import org.correomqtt.business.fileprovider.PersistSubscribeHistoryReadFailedEvent;
import org.correomqtt.business.fileprovider.PersistSubscribeHistoryWriteFailedEvent;
import org.correomqtt.business.fileprovider.SecretStoreErrorEvent;
import org.correomqtt.business.fileprovider.SettingsProvider;
import org.correomqtt.gui.helper.AlertHelper;
import org.correomqtt.gui.utils.PlatformUtils;

import java.util.ResourceBundle;

public class AlertController extends BaseControllerImpl implements
        ConfigObserver{

    public static final String ALERT_CONTROLLER_WARN_TITLE = "alertControllerWarnTitle";
    public static final String ALERT_EXCEPTION_TITLE = "Exception";

    private final ResourceBundle resources = ResourceBundle.getBundle("org.correomqtt.i18n", SettingsProvider.getInstance().getSettings().getCurrentLocale());
    private static AlertController instance;

    private AlertController() {
        EventBus.register(this);
        ConfigDispatcher.getInstance().addObserver(this);
        //TODO cleanup
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

    @SuppressWarnings("unused")
    public void errorReadingSubscriptionHistory(@Subscribe PersistSubscribeHistoryReadFailedEvent event) {
        PlatformUtils.runLaterIfNotInFxThread(() -> AlertHelper.warn(ALERT_EXCEPTION_TITLE,
                resources.getString("alertControllerErrorReadingSubscriptionHistoryContent") + event.throwable().getLocalizedMessage()
        ));
    }

    @SuppressWarnings("unused")
    @Subscribe
    public void errorReadingPublishHistory(PersistPublishHistoryReadFailedEvent event) {
        PlatformUtils.runLaterIfNotInFxThread(() -> AlertHelper.warn(ALERT_EXCEPTION_TITLE,
                resources.getString("alertControllerErrorReadingPublishHistoryContent") +
                        event.getException().getLocalizedMessage()
        ));
    }

    @SuppressWarnings("unused")
    @Subscribe
    public void errorWritingPublishHistory(PersistPublishHistoryWriteFailedEvent event) {
        PlatformUtils.runLaterIfNotInFxThread(() -> AlertHelper.warn(ALERT_EXCEPTION_TITLE,
                resources.getString("alertControllerErrorWritingPublishHistoryContent") +
                        event.getException().getLocalizedMessage()
        ));
    }


    @SuppressWarnings("unused")
    public void errorWritingSubscriptionHistory(@Subscribe PersistSubscribeHistoryWriteFailedEvent event) {
        PlatformUtils.runLaterIfNotInFxThread(() -> AlertHelper.warn(ALERT_EXCEPTION_TITLE,
                resources.getString("alertControllerErrorWritingSubscriptionHistoryContent") + event.throwable().getLocalizedMessage()
        ));
    }

    @Subscribe
    @SuppressWarnings("unused")
    public void onPasswordFileUnreadable(SecretStoreErrorEvent event) {
        if (event.error().equals(SecretStoreErrorEvent.Error.PASSWORD_FILE_UNREADABLE)) {
            PlatformUtils.runLaterIfNotInFxThread(() -> AlertHelper.warn(
                    resources.getString("onPasswordFileUnreadableFailedTitle"),
                    resources.getString("onPasswordFileUnreadableFailedContent"),
                    true
            ));
        }
    }
}