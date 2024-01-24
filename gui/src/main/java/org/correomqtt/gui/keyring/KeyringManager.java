package org.correomqtt.gui.keyring;

import org.apache.maven.artifact.versioning.ComparableVersion;
import org.correomqtt.core.fileprovider.EncryptionRecoverableException;
import org.correomqtt.core.fileprovider.SecretStoreProvider;
import org.correomqtt.core.keyring.Keyring;
import org.correomqtt.core.keyring.KeyringException;
import org.correomqtt.core.keyring.KeyringFactory;
import org.correomqtt.core.model.ConnectionConfigDTO;
import org.correomqtt.core.model.SettingsDTO;
import org.correomqtt.core.settings.SettingsManager;
import org.correomqtt.gui.utils.AlertHelper;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import javax.inject.Inject;
import java.util.List;
import java.util.ResourceBundle;
import java.util.UUID;
import java.util.stream.Collectors;

public class KeyringManager {

    private static final Logger LOGGER = LoggerFactory.getLogger(KeyringManager.class);

    private final ResourceBundle resources;
    private final SettingsManager settingsManager;
    private final SecretStoreProvider secretStoreProvider;
    private final AlertHelper alertHelper;
    private final KeyringFactory keyringFactory;
    private String masterPassword;
    private static final String KEYRING_LABEL = "CorreoMQTT_MasterPassword";
    private Keyring keyring;

    @Inject
    KeyringManager(KeyringFactory keyringFactory,
                   SettingsManager settingsManager,
                   SecretStoreProvider secretStoreProvider,
                   AlertHelper alertHelper) {
        this.keyringFactory = keyringFactory;
        resources = ResourceBundle.getBundle("org.correomqtt.i18n", settingsManager.getSettings().getCurrentLocale());
        this.settingsManager = settingsManager;
        this.secretStoreProvider = secretStoreProvider;
        this.alertHelper = alertHelper;
    }

    public void migrate(String newKeyringIdentifier) {

        retryWithMasterPassword(
                secretStoreProvider::ensurePasswordsAreDecrypted,
                resources.getString("onPasswordSaveFailedTitle"),
                resources.getString("onPasswordSaveFailedHeader"),
                resources.getString("onPasswordSaveFailedContent"),
                resources.getString("onPasswordSaveFailedGiveUp"),
                resources.getString("onPasswordSaveFailedTryAgain")
        );

        keyring = keyringFactory.createKeyringByIdentifier(newKeyringIdentifier);
        if (keyring == null) {
            alertHelper.warn(
                    resources.getString("couldNotCreateNewKeyringBackendTitle"),
                    resources.getString("couldNotCreateNewKeyringBackendContent")
            );
        } else {
            masterPassword = null;
            getMasterPassword();
            if (!keyring.requiresUserinput()) {
                keyring.setPassword(KEYRING_LABEL, masterPassword);
            }

            List<ConnectionConfigDTO> connections = settingsManager.getConnectionConfigs();
            retryWithMasterPassword(
                    pw -> settingsManager.saveConnections(connections, pw),
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
                masterPassword = alertHelper.passwordInput(
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
                retry = alertHelper.confirm(title, header, content, noButton, yesButton);
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
        SettingsDTO settings = settingsManager.getSettings();
        String oldKeyringIdentifier = settings.getKeyringIdentifier();

        keyring = null;

        if (oldKeyringIdentifier != null) {
            keyring = keyringFactory.createKeyringByIdentifier(oldKeyringIdentifier);
            if (keyring == null) {
                LOGGER.info("Configured keyring {} not found.", oldKeyringIdentifier);
            } else {
                LOGGER.info("Configured keyring {} found.", oldKeyringIdentifier);
            }
        }

        if (keyring == null) {
            List<Keyring> keyrings = keyringFactory.create(); // Not null, will produce UserInputKeyring for sure
            if (LOGGER.isInfoEnabled()) {
                LOGGER.info("Detected keyrings: {}", keyrings.stream().map(Keyring::getName).collect(Collectors.joining(", ")));
            }
            if (keyrings.size() <= 2) {
                keyring = keyrings.get(0);
            } else {
                keyring = alertHelper.select("Multiple KeyringsFound", "Select a keyring", keyrings); //TODO
            }
        }

        if (keyring == null) {
            throw new KeyringException("No supported keyring backend found.");
        }

        String newKeyringIdentifier = keyring.getIdentifier();

        ComparableVersion createdVersion = new ComparableVersion(settings.getConfigCreatedWithCorreoVersion().replaceAll("[^0-9.]", ""));
        ComparableVersion keyringSupportVersion = new ComparableVersion("0.13.0");

        if (oldKeyringIdentifier == null && keyringSupportVersion.compareTo(createdVersion) < 0) {
            alertHelper.info(
                    resources.getString("newKeyringTitle"),
                    resources.getString("newKeyringContent") + newKeyringIdentifier,
                    true
            );
        } else if (!newKeyringIdentifier.equals(oldKeyringIdentifier)) {
            alertHelper.warn(
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
        secretStoreProvider.wipe();
    }
}
