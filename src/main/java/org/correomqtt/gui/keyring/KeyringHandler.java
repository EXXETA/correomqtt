package org.correomqtt.gui.keyring;

import org.apache.maven.artifact.versioning.ComparableVersion;
import org.correomqtt.business.keyring.Keyring;
import org.correomqtt.business.keyring.KeyringFactory;
import org.correomqtt.business.model.ConnectionConfigDTO;
import org.correomqtt.business.model.SettingsDTO;
import org.correomqtt.business.provider.EncryptionRecoverableException;
import org.correomqtt.business.provider.SecretStoreProvider;
import org.correomqtt.business.provider.SettingsProvider;
import org.correomqtt.gui.helper.AlertHelper;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.List;
import java.util.ResourceBundle;
import java.util.UUID;

public class KeyringHandler {

    private static final Logger LOGGER = LoggerFactory.getLogger(KeyringHandler.class);

    private static KeyringHandler instance = null;
    private ResourceBundle resources;
    private String masterPassword;
    private static final String KEYRING_LABEL = "CorreoMQTT_MasterPassword";
    private Keyring keyring;

    public static synchronized KeyringHandler getInstance() {
        if (instance == null) {
            instance = new KeyringHandler();
            return instance;
        } else {
            return instance;
        }
    }

    private KeyringHandler() {
        resources = ResourceBundle.getBundle("org.correomqtt.i18n", SettingsProvider.getInstance().getSettings().getCurrentLocale());
    }

    public void migrate(String newKeyringIdentifier) {

        retryWithMasterPassword(
                pw -> SecretStoreProvider.getInstance().ensurePasswordsAreDecrypted(pw),
                resources.getString("onPasswordSaveFailedTitle"),
                resources.getString("onPasswordSaveFailedHeader"),
                resources.getString("onPasswordSaveFailedContent"),
                resources.getString("onPasswordSaveFailedGiveUp"),
                resources.getString("onPasswordSaveFailedTryAgain")
        );

        keyring = KeyringFactory.createKeyringByIdentifier(newKeyringIdentifier);
        if (keyring == null) {
            AlertHelper.warn(
                    resources.getString("couldNotCreateNewKeyringBackendTitle"),
                    resources.getString("couldNotCreateNewKeyringBackendContent")
            );
        } else {
            masterPassword = null;
            getMasterPassword();
            if (!keyring.requiresUserinput()) {
                keyring.setPassword(KEYRING_LABEL, masterPassword);
            }

            List<ConnectionConfigDTO> connections = SettingsProvider.getInstance().getConnectionConfigs();
            retryWithMasterPassword(
                    pw -> SettingsProvider.getInstance().saveConnections(connections, pw),
                    resources.getString("onPasswordSaveFailedTitle"),
                    resources.getString("onPasswordSaveFailedHeader"),
                    resources.getString("onPasswordSaveFailedContent"),
                    resources.getString("onPasswordSaveFailedGiveUp"),
                    resources.getString("onPasswordSaveFailedTryAgain")
            );
        }

    }

    public String getMasterPassword() {

        if (masterPassword != null) {
            return masterPassword;
        }

        if (keyring != null) {
            if (keyring.requiresUserinput()) {
                masterPassword = AlertHelper.passwordInput(
                        resources.getString("onPasswordRequiredTitle"),
                        resources.getString("onPasswordRequiredHeader"),
                        resources.getString("onPasswordRequiredContent")
                );
            } else {
                masterPassword = keyring.getPassword(KEYRING_LABEL);
                if (masterPassword == null || masterPassword.isEmpty()) {
                    keyring.setPassword(KEYRING_LABEL, UUID.randomUUID().toString());
                    masterPassword = keyring.getPassword(KEYRING_LABEL);
                }
            }
        }

        // empty check?

        return masterPassword;
    }

    public void retryWithMasterPassword(ConsumerWithRetry consumer, String title, String header, String content, String noButton, String yesButton) {
        boolean failed = false;
        do {
            boolean retry = false;
            if (failed) {
                this.masterPassword = null;
                retry = AlertHelper.confirm(title, header, content, noButton, yesButton);
            }
            if (!failed || retry) {
                failed = false;
                try {
                    consumer.apply(getMasterPassword());
                } catch (EncryptionRecoverableException e) {
                    LOGGER.error("Error de/encrypt passwords. ", e);
                    failed = true;
                }
            } else {
                failed = false; //no retry
                wipe();
            }
        } while (failed);
    }

    public void init() {
        SettingsDTO settings = SettingsProvider.getInstance().getSettings();
        String oldKeyringIdentifier = settings.getKeyringIdentifier();
        keyring = null;

        if (oldKeyringIdentifier != null) {
            keyring = KeyringFactory.createKeyringByIdentifier(oldKeyringIdentifier);
        }

        if (keyring == null) {
            keyring = KeyringFactory.create(); // Not null, will produce UserInputKeyring for sure
        }

        String newKeyringIdentifier = keyring.getIdentifier();

        ComparableVersion createdVersion = new ComparableVersion(settings.getConfigCreatedWithCorreoVersion().replaceAll("[^0-9\\.]", ""));
        ComparableVersion keyringSupportVersion = new ComparableVersion("0.13.0");

        if (oldKeyringIdentifier == null && keyringSupportVersion.compareTo(createdVersion) > 0) {
            AlertHelper.info(
                    resources.getString("newKeyringTitle"),
                    resources.getString("newKeyringContent") + newKeyringIdentifier,
                    true
            );
        } else if (!newKeyringIdentifier.equals(oldKeyringIdentifier)) {
            AlertHelper.warn(
                    resources.getString("changedKeyringTitle"),
                    resources.getString("changedKeyringContent") + oldKeyringIdentifier + " -> " + newKeyringIdentifier,
                    true
            );
        }

        if (!newKeyringIdentifier.equals(oldKeyringIdentifier)) {
            settings.setKeyringIdentifier(newKeyringIdentifier); // This is called during init phase, so no need to save here.
        }

    }

    public void wipe() {
        masterPassword = null;
        SecretStoreProvider.getInstance().wipe();
    }
}
