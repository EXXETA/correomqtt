package org.correomqtt.gui.views;

import org.correomqtt.core.eventbus.EventBus;
import org.correomqtt.core.eventbus.Subscribe;
import org.correomqtt.core.fileprovider.DirectoryCanNotBeCreatedEvent;
import org.correomqtt.core.fileprovider.InvalidConfigFileEvent;
import org.correomqtt.core.fileprovider.InvalidHooksFileEvent;
import org.correomqtt.core.fileprovider.InvalidPasswordFileEvent;
import org.correomqtt.core.fileprovider.PersistPublishHistoryReadFailedEvent;
import org.correomqtt.core.fileprovider.PersistPublishHistoryWriteFailedEvent;
import org.correomqtt.core.fileprovider.PersistSubscribeHistoryReadFailedEvent;
import org.correomqtt.core.fileprovider.PersistSubscribeHistoryWriteFailedEvent;
import org.correomqtt.core.settings.SettingsProvider;
import org.correomqtt.core.fileprovider.SettingsUpdatedEvent;
import org.correomqtt.core.fileprovider.UnaccessibleConfigFileEvent;
import org.correomqtt.core.fileprovider.UnaccessiblePasswordFileEvent;
import org.correomqtt.core.fileprovider.UserHomeNull;
import org.correomqtt.core.fileprovider.WindowsAppDataNullEvent;
import org.correomqtt.gui.theme.ThemeManager;
import org.correomqtt.gui.utils.AlertHelper;
import org.correomqtt.gui.utils.PlatformUtils;
import org.correomqtt.gui.views.base.BaseControllerImpl;

import javax.inject.Inject;
import java.text.MessageFormat;
import java.util.ResourceBundle;

public class AlertController extends BaseControllerImpl {

    public static final String ALERT_CONTROLLER_WARN_TITLE = "alertControllerWarnTitle";
    public static final String ALERT_EXCEPTION_TITLE = "Exception";

    private final ResourceBundle resources;
    private final AlertHelper alertHelper;

    @Inject
    AlertController(SettingsProvider settingsProvider,
                    ThemeManager themeManager,
                    AlertHelper alertHelper) {
        super(settingsProvider, themeManager);
        this.alertHelper = alertHelper;
        resources = ResourceBundle.getBundle("org.correomqtt.i18n", settingsProvider.getSettings().getCurrentLocale());
    }

    public void activate() {
        EventBus.register(this);
    }

    public void deactivate() {
        EventBus.unregister(this);
    }

    @SuppressWarnings("unused")
    public void onConfigDirectoryEmpty(@Subscribe DirectoryCanNotBeCreatedEvent event) {
        PlatformUtils.runLaterIfNotInFxThread(() -> alertHelper.warn(resources.getString(ALERT_CONTROLLER_WARN_TITLE),
                MessageFormat.format(resources.getString("alertControllerOnConfigDirectoryEmptyContent"), event.path())
        ));
    }

    @SuppressWarnings("unused")
    @Subscribe(UnaccessibleConfigFileEvent.class)
    public void onConfigDirectoryNotAccessible() {
        PlatformUtils.runLaterIfNotInFxThread(() -> alertHelper.warn(resources.getString(ALERT_CONTROLLER_WARN_TITLE),
                resources.getString("alertControllerOnConfigDirectoryNotAccessibleContent")
        ));
    }

    @SuppressWarnings("unused")
    @Subscribe(UnaccessiblePasswordFileEvent.class)
    public void onPasswordDirectoryNotAccessible() {
        PlatformUtils.runLaterIfNotInFxThread(() -> alertHelper.warn(resources.getString(ALERT_CONTROLLER_WARN_TITLE),
                resources.getString("alertControllerOnPasswordDirectoryNotAccessibleContent")
        ));
    }

    @SuppressWarnings("unused")
    @Subscribe(UnaccessiblePasswordFileEvent.class)
    public void onHooksDirectoryNotAccessible() {
        PlatformUtils.runLaterIfNotInFxThread(() -> alertHelper.warn(resources.getString(ALERT_CONTROLLER_WARN_TITLE),
                resources.getString("alertControllerOnPasswordDirectoryNotAccessibleContent")
        ));
    }

    @SuppressWarnings("unused")
    @Subscribe(WindowsAppDataNullEvent.class)
    public void onAppDataNull() {
        PlatformUtils.runLaterIfNotInFxThread(() -> alertHelper.warn(resources.getString(ALERT_CONTROLLER_WARN_TITLE),
                resources.getString("alertControllerOnAppDataNullContent")
        ));
    }

    @SuppressWarnings("unused")
    @Subscribe(UserHomeNull.class)
    public void onUserHomeNull() {
        PlatformUtils.runLaterIfNotInFxThread(() -> alertHelper.warn(resources.getString(ALERT_CONTROLLER_WARN_TITLE),
                resources.getString("alertControllerOnUserHomeNullContent")
        ));
    }


    @SuppressWarnings("unused")
    @Subscribe(InvalidConfigFileEvent.class)
    public void onInvalidConfigJsonFormat() {
        PlatformUtils.runLaterIfNotInFxThread(() -> alertHelper.warn(resources.getString("alertControllerOnInvalidJsonFormatTitle"),
                resources.getString("alertControllerOnInvalidJsonFormatContent")
        ));
    }


    @SuppressWarnings("unused")
    @Subscribe(InvalidPasswordFileEvent.class)
    public void onInvalidPasswordJsonFormat() {
        PlatformUtils.runLaterIfNotInFxThread(() -> alertHelper.warn(
                resources.getString("onPasswordFileUnreadableFailedTitle"),
                resources.getString("onPasswordFileUnreadableFailedContent"),
                true
        ));

    }


    @SuppressWarnings("unused")
    @Subscribe(InvalidHooksFileEvent.class)
    public void onInvalidHooksJsonFormat() {
        PlatformUtils.runLaterIfNotInFxThread(() -> alertHelper.warn(resources.getString("alertControllerOnInvalidJsonFormatTitle"),
                resources.getString("alertControllerOnInvalidJsonFormatContent")
        ));
    }

    @SuppressWarnings("unused")
    public void onSettingsUpdated(@Subscribe SettingsUpdatedEvent event) {
        if (event.restartRequired()) {
            PlatformUtils.runLaterIfNotInFxThread(() -> alertHelper.info(resources.getString("alertControllerOnSettingsUpdatedTitle"),
                    resources.getString("alertControllerOnSettingsUpdatedContent")
            ));
        }
    }

    @SuppressWarnings("unused")
    public void errorReadingSubscriptionHistory(@Subscribe PersistSubscribeHistoryReadFailedEvent event) {
        PlatformUtils.runLaterIfNotInFxThread(() -> alertHelper.warn(ALERT_EXCEPTION_TITLE,
                resources.getString("alertControllerErrorReadingSubscriptionHistoryContent") + event.throwable().getLocalizedMessage()
        ));
    }

    @SuppressWarnings("unused")
    @Subscribe
    public void errorReadingPublishHistory(PersistPublishHistoryReadFailedEvent event) {
        PlatformUtils.runLaterIfNotInFxThread(() -> alertHelper.warn(ALERT_EXCEPTION_TITLE,
                resources.getString("alertControllerErrorReadingPublishHistoryContent") +
                        event.getException().getLocalizedMessage()
        ));
    }

    @SuppressWarnings("unused")
    @Subscribe
    public void errorWritingPublishHistory(PersistPublishHistoryWriteFailedEvent event) {
        PlatformUtils.runLaterIfNotInFxThread(() -> alertHelper.warn(ALERT_EXCEPTION_TITLE,
                resources.getString("alertControllerErrorWritingPublishHistoryContent") +
                        event.getException().getLocalizedMessage()
        ));
    }

    @SuppressWarnings("unused")
    public void errorWritingSubscriptionHistory(@Subscribe PersistSubscribeHistoryWriteFailedEvent event) {
        PlatformUtils.runLaterIfNotInFxThread(() -> alertHelper.warn(ALERT_EXCEPTION_TITLE,
                resources.getString("alertControllerErrorWritingSubscriptionHistoryContent") + event.throwable().getLocalizedMessage()
        ));
    }
}