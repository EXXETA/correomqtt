package com.exxeta.correomqtt.gui.controller;

import com.exxeta.correomqtt.business.dispatcher.ConfigDispatcher;
import com.exxeta.correomqtt.business.dispatcher.ConfigObserver;
import com.exxeta.correomqtt.business.dispatcher.PersistPublishHistoryObserver;
import com.exxeta.correomqtt.business.dispatcher.PersistSubscriptionHistoryObserver;
import com.exxeta.correomqtt.business.services.ConfigService;
import com.exxeta.correomqtt.gui.helper.AlertHelper;

import java.util.ResourceBundle;

public class AlertController extends BaseController implements
        ConfigObserver,
        PersistPublishHistoryObserver,
        PersistSubscriptionHistoryObserver {

    private ResourceBundle resources = ResourceBundle.getBundle("com.exxeta.correomqtt.i18n", ConfigService.getInstance().getSettings().getCurrentLocale());
    private static AlertController instance;

    private AlertController() {
        ConfigDispatcher.getInstance().addObserver(this);
    }

    public static void activate() {
        if (instance == null) {
            instance = new AlertController();
        }
    }

    @Override
    public void onConfigDirectoryEmpty() {
        AlertHelper.warn(resources.getString("alertControllerWarnTitle"),
                resources.getString("alertControllerOnConfigDirectoryEmptyContent"));
    }

    @Override
    public void onConfigDirectoryNotAccessible() {
        AlertHelper.warn(resources.getString("alertControllerWarnTitle"),
                resources.getString("alertControllerOnConfigDirectoryNotAccessibleContent"));
    }

    @Override
    public void onAppDataNull() {
        AlertHelper.warn(resources.getString("alertControllerWarnTitle"),
                resources.getString("alertControllerOnAppDataNullContent"));
    }

    @Override
    public void onUserHomeNull() {
        AlertHelper.warn(resources.getString("alertControllerWarnTitle"),
                resources.getString("alertControllerOnUserHomeNullContent"));
    }

    @Override
    public void onFileAlreadyExists() {
        AlertHelper.warn(resources.getString("alertControllerOnFileAlreadyExistsTitle"),
                resources.getString("alertControllerOnFileAlreadyExistsContent"));
    }

    @Override
    public void onInvalidPath() {
        AlertHelper.warn(resources.getString("alertControllerOnInvalidPathTitle"),
                resources.getString("alertControllerOnInvalidPathContent"));
    }

    @Override
    public void onInvalidJsonFormat() {
        AlertHelper.warn(resources.getString("alertControllerOnInvalidJsonFormatTitle"),
                resources.getString("alertControllerOnInvalidJsonFormatContent"));
    }

    @Override
    public void onSavingFailed() {
        AlertHelper.warn(resources.getString("alertControllerOnSavingFailedTitle"),
                resources.getString("alertControllerOnSavingFailedContent"));
    }

    @Override
    public void onConnectionsUpdated() {

    }

    @Override
    public void onConfigPrepareFailed() {
        AlertHelper.warn("Exception",
                resources.getString("alertControllerOnConfigPrepareFailedContent"));
    }

    @Override
    public void onSettingsUpdated() {
        AlertHelper.info(resources.getString("alertControllerOnSettingsUpdatedTitle"),
                resources.getString("alertControllerOnSettingsUpdatedContent"));
    }

    @Override
    public void updateSubscriptions(String connectionId) {
        // nothing to do
    }

    @Override
    public void errorReadingSubscriptionHistory(Throwable exception) {
        AlertHelper.warn("Exception",
                resources.getString("alertControllerErrorReadingSubscriptionHistoryContent") + exception.getLocalizedMessage());
    }

    @Override
    public String getConnectionId() {
        return null;
    }

    @Override
    public void errorReadingPublishHistory(Throwable exception) {
        AlertHelper.warn("Exception",
                resources.getString("alertControllerErrorReadingPublishHistoryContent") + exception.getLocalizedMessage());
    }

    @Override
    public void errorWritingPublishHistory(Throwable exception) {
        AlertHelper.warn("Exception",
                resources.getString("alertControllerErrorWritingPublishHistoryContent") + exception.getLocalizedMessage());
    }

    @Override
    public void updatedPublishes(String connectionId) {

    }

    @Override
    public void errorWritingSubscriptionHistory(Throwable exception) {
        AlertHelper.warn("Exception",
                resources.getString("alertControllerErrorWritingSubscriptionHistoryContent") + exception.getLocalizedMessage());
    }
}