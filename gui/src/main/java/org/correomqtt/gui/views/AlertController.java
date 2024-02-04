package org.correomqtt.gui.views;

import org.correomqtt.core.CoreManager;
import org.correomqtt.core.fileprovider.DirectoryCanNotBeCreatedEvent;
import org.correomqtt.core.fileprovider.InvalidConfigFileEvent;
import org.correomqtt.core.fileprovider.InvalidHooksFileEvent;
import org.correomqtt.core.fileprovider.InvalidPasswordFileEvent;
import org.correomqtt.core.fileprovider.PersistPublishHistoryReadFailedEvent;
import org.correomqtt.core.fileprovider.PersistPublishHistoryWriteFailedEvent;
import org.correomqtt.core.fileprovider.PersistSubscribeHistoryReadFailedEvent;
import org.correomqtt.core.fileprovider.PersistSubscribeHistoryWriteFailedEvent;
import org.correomqtt.core.fileprovider.SettingsUpdatedEvent;
import org.correomqtt.core.fileprovider.UnaccessibleConfigFileEvent;
import org.correomqtt.core.fileprovider.UnaccessiblePasswordFileEvent;
import org.correomqtt.core.fileprovider.UserHomeNull;
import org.correomqtt.core.fileprovider.WindowsAppDataNullEvent;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import org.correomqtt.di.Observes;
import org.correomqtt.gui.theme.ThemeManager;
import org.correomqtt.gui.utils.AlertHelper;
import org.correomqtt.gui.utils.PlatformUtils;
import org.correomqtt.gui.views.base.BaseControllerImpl;

import java.text.MessageFormat;
import java.util.ResourceBundle;

@DefaultBean
public class AlertController extends BaseControllerImpl {

    public static final String ALERT_CONTROLLER_WARN_TITLE = "alertControllerWarnTitle";
    public static final String ALERT_EXCEPTION_TITLE = "Exception";

    private final ResourceBundle resources;
    private final AlertHelper alertHelper;

    @Inject
    AlertController(CoreManager coreManager,
                    ThemeManager themeManager,
                    AlertHelper alertHelper) {
        super(coreManager, themeManager);
        this.alertHelper = alertHelper;
        resources = ResourceBundle.getBundle("org.correomqtt.i18n", coreManager.getSettingsManager().getSettings().getCurrentLocale());
    }

    @SuppressWarnings("unused")
    public void onConfigDirectoryEmpty(@Observes(autocreate = true) DirectoryCanNotBeCreatedEvent event) {
        PlatformUtils.runLaterIfNotInFxThread(() -> alertHelper.warn(resources.getString(ALERT_CONTROLLER_WARN_TITLE),
                MessageFormat.format(resources.getString("alertControllerOnConfigDirectoryEmptyContent"), event.path())
        ));
    }

    @SuppressWarnings("unused")
    @Observes(autocreate = true, value = UnaccessibleConfigFileEvent.class)
    public void onConfigDirectoryNotAccessible() {
        PlatformUtils.runLaterIfNotInFxThread(() -> alertHelper.warn(resources.getString(ALERT_CONTROLLER_WARN_TITLE),
                resources.getString("alertControllerOnConfigDirectoryNotAccessibleContent")
        ));
    }

    @SuppressWarnings("unused")
    @Observes(autocreate = true, value = UnaccessiblePasswordFileEvent.class)
    public void onPasswordDirectoryNotAccessible() {
        PlatformUtils.runLaterIfNotInFxThread(() -> alertHelper.warn(resources.getString(ALERT_CONTROLLER_WARN_TITLE),
                resources.getString("alertControllerOnPasswordDirectoryNotAccessibleContent")
        ));
    }

    @SuppressWarnings("unused")
    @Observes(autocreate = true, value = UnaccessiblePasswordFileEvent.class)
    public void onHooksDirectoryNotAccessible() {
        PlatformUtils.runLaterIfNotInFxThread(() -> alertHelper.warn(resources.getString(ALERT_CONTROLLER_WARN_TITLE),
                resources.getString("alertControllerOnPasswordDirectoryNotAccessibleContent")
        ));
    }

    @SuppressWarnings("unused")
    @Observes(autocreate = true, value = WindowsAppDataNullEvent.class)
    public void onAppDataNull() {
        PlatformUtils.runLaterIfNotInFxThread(() -> alertHelper.warn(resources.getString(ALERT_CONTROLLER_WARN_TITLE),
                resources.getString("alertControllerOnAppDataNullContent")
        ));
    }

    @SuppressWarnings("unused")
    @Observes(autocreate = true, value = UserHomeNull.class)
    public void onUserHomeNull() {
        PlatformUtils.runLaterIfNotInFxThread(() -> alertHelper.warn(resources.getString(ALERT_CONTROLLER_WARN_TITLE),
                resources.getString("alertControllerOnUserHomeNullContent")
        ));
    }

    @SuppressWarnings("unused")
    @Observes(autocreate = true, value = InvalidConfigFileEvent.class)
    public void onInvalidConfigJsonFormat() {
        PlatformUtils.runLaterIfNotInFxThread(() -> alertHelper.warn(resources.getString("alertControllerOnInvalidJsonFormatTitle"),
                resources.getString("alertControllerOnInvalidJsonFormatContent")
        ));
    }

    @SuppressWarnings("unused")
    @Observes(autocreate = true, value = InvalidPasswordFileEvent.class)
    public void onInvalidPasswordJsonFormat() {
        PlatformUtils.runLaterIfNotInFxThread(() -> alertHelper.warn(
                resources.getString("onPasswordFileUnreadableFailedTitle"),
                resources.getString("onPasswordFileUnreadableFailedContent"),
                true
        ));
    }

    @SuppressWarnings("unused")
    @Observes(autocreate = true, value = InvalidHooksFileEvent.class)
    public void onInvalidHooksJsonFormat() {
        PlatformUtils.runLaterIfNotInFxThread(() -> alertHelper.warn(resources.getString("alertControllerOnInvalidJsonFormatTitle"),
                resources.getString("alertControllerOnInvalidJsonFormatContent")
        ));
    }

    @SuppressWarnings("unused")
    public void onSettingsUpdated(@Observes(autocreate = true) SettingsUpdatedEvent event) {
        if (event.restartRequired()) {
            PlatformUtils.runLaterIfNotInFxThread(() -> alertHelper.info(resources.getString("alertControllerOnSettingsUpdatedTitle"),
                    resources.getString("alertControllerOnSettingsUpdatedContent")
            ));
        }
    }

    @SuppressWarnings("unused")
    public void errorReadingSubscriptionHistory(@Observes(autocreate = true) PersistSubscribeHistoryReadFailedEvent event) {
        PlatformUtils.runLaterIfNotInFxThread(() -> alertHelper.warn(ALERT_EXCEPTION_TITLE,
                resources.getString("alertControllerErrorReadingSubscriptionHistoryContent") + event.throwable().getLocalizedMessage()
        ));
    }

    @SuppressWarnings("unused")
    public void errorReadingPublishHistory(@Observes(autocreate = true) PersistPublishHistoryReadFailedEvent event) {
        PlatformUtils.runLaterIfNotInFxThread(() -> alertHelper.warn(ALERT_EXCEPTION_TITLE,
                resources.getString("alertControllerErrorReadingPublishHistoryContent") +
                        event.getException().getLocalizedMessage()
        ));
    }

    @SuppressWarnings("unused")
    public void errorWritingPublishHistory(@Observes(autocreate = true) PersistPublishHistoryWriteFailedEvent event) {
        PlatformUtils.runLaterIfNotInFxThread(() -> alertHelper.warn(ALERT_EXCEPTION_TITLE,
                resources.getString("alertControllerErrorWritingPublishHistoryContent") +
                        event.getException().getLocalizedMessage()
        ));
    }

    @SuppressWarnings("unused")
    public void errorWritingSubscriptionHistory(@Observes(autocreate = true) PersistSubscribeHistoryWriteFailedEvent event) {
        PlatformUtils.runLaterIfNotInFxThread(() -> alertHelper.warn(ALERT_EXCEPTION_TITLE,
                resources.getString("alertControllerErrorWritingSubscriptionHistoryContent") + event.throwable().getLocalizedMessage()
        ));
    }
}