package org.correomqtt.gui.views;

import org.correomqtt.business.eventbus.EventBus;
import org.correomqtt.business.eventbus.Subscribe;
import org.correomqtt.business.fileprovider.DirectoryCanNotBeCreatedEvent;
import org.correomqtt.business.fileprovider.InvalidConfigFileEvent;
import org.correomqtt.business.fileprovider.InvalidHooksFileEvent;
import org.correomqtt.business.fileprovider.InvalidPasswordFileEvent;
import org.correomqtt.business.fileprovider.PersistPublishHistoryReadFailedEvent;
import org.correomqtt.business.fileprovider.PersistPublishHistoryWriteFailedEvent;
import org.correomqtt.business.fileprovider.PersistSubscribeHistoryReadFailedEvent;
import org.correomqtt.business.fileprovider.PersistSubscribeHistoryWriteFailedEvent;
import org.correomqtt.business.fileprovider.SettingsProvider;
import org.correomqtt.business.fileprovider.SettingsUpdatedEvent;
import org.correomqtt.business.fileprovider.UnaccessibleConfigFileEvent;
import org.correomqtt.business.fileprovider.UnaccessiblePasswordFileEvent;
import org.correomqtt.business.fileprovider.UserHomeNull;
import org.correomqtt.business.fileprovider.WindowsAppDataNullEvent;
import org.correomqtt.gui.utils.AlertHelper;
import org.correomqtt.gui.utils.PlatformUtils;
import org.correomqtt.gui.views.base.BaseControllerImpl;

import java.text.MessageFormat;
import java.util.ResourceBundle;

public class AlertController extends BaseControllerImpl {

    public static final String ALERT_CONTROLLER_WARN_TITLE = "alertControllerWarnTitle";
    public static final String ALERT_EXCEPTION_TITLE = "Exception";

    private final ResourceBundle resources = ResourceBundle.getBundle("org.correomqtt.i18n", SettingsProvider.getInstance().getSettings().getCurrentLocale());
    private static AlertController instance;

    private AlertController() {
        EventBus.register(this);
    }

    public static void activate() {
        if (instance == null) {
            instance = new AlertController();
        }
    }

    public static void deactivate() {
        if(instance != null){
            instance.cleanup();
        }
    }

    private void cleanup() {
        EventBus.unregister(this);
    }

    @SuppressWarnings("unused")
    public void onConfigDirectoryEmpty(@Subscribe DirectoryCanNotBeCreatedEvent event) {
        PlatformUtils.runLaterIfNotInFxThread(() -> AlertHelper.warn(resources.getString(ALERT_CONTROLLER_WARN_TITLE),
                MessageFormat.format(resources.getString("alertControllerOnConfigDirectoryEmptyContent"), event.path())
        ));
    }

    @SuppressWarnings("unused")
    @Subscribe(UnaccessibleConfigFileEvent.class)
    public void onConfigDirectoryNotAccessible() {
        PlatformUtils.runLaterIfNotInFxThread(() -> AlertHelper.warn(resources.getString(ALERT_CONTROLLER_WARN_TITLE),
                resources.getString("alertControllerOnConfigDirectoryNotAccessibleContent")
        ));
    }

    @SuppressWarnings("unused")
    @Subscribe(UnaccessiblePasswordFileEvent.class)
    public void onPasswordDirectoryNotAccessible() {
        PlatformUtils.runLaterIfNotInFxThread(() -> AlertHelper.warn(resources.getString(ALERT_CONTROLLER_WARN_TITLE),
                resources.getString("alertControllerOnPasswordDirectoryNotAccessibleContent")
        ));
    }

    @SuppressWarnings("unused")
    @Subscribe(UnaccessiblePasswordFileEvent.class)
    public void onHooksDirectoryNotAccessible() {
        PlatformUtils.runLaterIfNotInFxThread(() -> AlertHelper.warn(resources.getString(ALERT_CONTROLLER_WARN_TITLE),
                resources.getString("alertControllerOnPasswordDirectoryNotAccessibleContent")
        ));
    }

    @SuppressWarnings("unused")
    @Subscribe(WindowsAppDataNullEvent.class)
    public void onAppDataNull() {
        PlatformUtils.runLaterIfNotInFxThread(() -> AlertHelper.warn(resources.getString(ALERT_CONTROLLER_WARN_TITLE),
                resources.getString("alertControllerOnAppDataNullContent")
        ));
    }

    @SuppressWarnings("unused")
    @Subscribe(UserHomeNull.class)
    public void onUserHomeNull() {
        PlatformUtils.runLaterIfNotInFxThread(() -> AlertHelper.warn(resources.getString(ALERT_CONTROLLER_WARN_TITLE),
                resources.getString("alertControllerOnUserHomeNullContent")
        ));
    }


    @SuppressWarnings("unused")
    @Subscribe(InvalidConfigFileEvent.class)
    public void onInvalidConfigJsonFormat() {
        PlatformUtils.runLaterIfNotInFxThread(() -> AlertHelper.warn(resources.getString("alertControllerOnInvalidJsonFormatTitle"),
                resources.getString("alertControllerOnInvalidJsonFormatContent")
        ));
    }


    @SuppressWarnings("unused")
    @Subscribe(InvalidPasswordFileEvent.class)
    public void onInvalidPasswordJsonFormat() {
        PlatformUtils.runLaterIfNotInFxThread(() -> AlertHelper.warn(
                resources.getString("onPasswordFileUnreadableFailedTitle"),
                resources.getString("onPasswordFileUnreadableFailedContent"),
                true
        ));

    }


    @SuppressWarnings("unused")
    @Subscribe(InvalidHooksFileEvent.class)
    public void onInvalidHooksJsonFormat() {
        PlatformUtils.runLaterIfNotInFxThread(() -> AlertHelper.warn(resources.getString("alertControllerOnInvalidJsonFormatTitle"),
                resources.getString("alertControllerOnInvalidJsonFormatContent")
        ));
    }

    @SuppressWarnings("unused")
    public void onSettingsUpdated(@Subscribe SettingsUpdatedEvent event) {
        if (event.restartRequired()) {
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
}