package org.correomqtt.gui.keyring;

import org.correomqtt.business.keyring.Keyring;
import org.correomqtt.business.keyring.KeyringFactory;
import org.correomqtt.business.model.ConnectionConfigDTO;
import org.correomqtt.business.model.SettingsDTO;
import org.correomqtt.business.provider.PasswordRecoverableException;
import org.correomqtt.business.provider.SecretStoreProvider;
import org.correomqtt.business.provider.SettingsProvider;
import org.correomqtt.gui.helper.AlertHelper;

import java.util.List;
import java.util.ResourceBundle;
import java.util.UUID;

public class KeyringHandler {

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
                masterPassword -> SecretStoreProvider.getInstance().ensurePasswordsAreDecrypted(masterPassword),
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
                    resources.getString("couldNotCreateNewKeyringBackendContent") + keyring.getIdentifier()
            );
        } else {
            masterPassword = null;
            getMasterPassword();
            if(!keyring.requiresUserinput()) {
                keyring.setPassword(KEYRING_LABEL, masterPassword);
            }

            List<ConnectionConfigDTO> connections = SettingsProvider.getInstance().getConnectionConfigs();
            retryWithMasterPassword(
                    masterPassword -> SettingsProvider.getInstance().saveConnections(connections, masterPassword),
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
                } catch (PasswordRecoverableException e) {
                    failed = true;
                }
            } else {
                failed = false; //no retry

                try {
                    SecretStoreProvider.getInstance().wipe(getMasterPassword());
                } catch (PasswordRecoverableException e) {
                    AlertHelper.warn(
                            resources.getString("couldNotContinueWithoutPasswordsTitle"),
                            resources.getString("couldNotContinueWithoutPasswordsContent")
                    );
                }

            }
        } while (failed);
    }

    public void init() {
        SettingsDTO settings = SettingsProvider.getInstance().getSettings();
        String oldKeyringIdentifier = settings.getKeyringIdentifier();
        Keyring keyring = null;

        if (oldKeyringIdentifier != null) {
            keyring = KeyringFactory.createKeyringByIdentifier(oldKeyringIdentifier);
        }

        if (keyring == null) {
            keyring = KeyringFactory.create();
        }

        String newKeyringIdentifier = keyring.getIdentifier();

        if (oldKeyringIdentifier == null) {
            AlertHelper.info(
                    resources.getString("newKeyringTitle"),
                    resources.getString("newKeyringContent") + newKeyringIdentifier,
                    true
            );
        } else if (!oldKeyringIdentifier.equals(newKeyringIdentifier)) {
            AlertHelper.info(
                    resources.getString("changedKeyringTitle"),
                    resources.getString("changedKeyringContent") + oldKeyringIdentifier + " -> " + newKeyringIdentifier,
                    true
            );
        }

        this.keyring = keyring;
    }
}
