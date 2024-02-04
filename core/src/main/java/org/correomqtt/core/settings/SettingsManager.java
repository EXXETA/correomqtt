package org.correomqtt.core.settings;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.correomqtt.di.SoyEvents;
import org.correomqtt.core.exception.CorreoMqttConfigurationMissingException;
import org.correomqtt.core.fileprovider.BaseUserFileProvider;
import org.correomqtt.core.fileprovider.ConfigSaveFailedEvent;
import org.correomqtt.core.fileprovider.ConnectionsUpdatedEvent;
import org.correomqtt.core.fileprovider.EncryptionRecoverableException;
import org.correomqtt.core.fileprovider.InvalidConfigFileEvent;
import org.correomqtt.core.fileprovider.SecretStoreProvider;
import org.correomqtt.core.fileprovider.SettingsUpdatedEvent;
import org.correomqtt.core.fileprovider.UnaccessibleConfigFileEvent;
import org.correomqtt.core.model.ConfigDTO;
import org.correomqtt.core.model.ConnectionConfigDTO;
import org.correomqtt.core.model.MessageListViewConfig;
import org.correomqtt.core.model.SettingsDTO;
import org.correomqtt.core.model.ThemeSettingsDTO;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import org.correomqtt.di.Inject;
import org.correomqtt.di.SingletonBean;
import java.io.IOException;
import java.nio.file.InvalidPathException;
import java.util.HashSet;
import java.util.List;
import java.util.Set;

import static org.correomqtt.core.model.ConnectionPasswordType.AUTH_PASSWORD;
import static org.correomqtt.core.model.ConnectionPasswordType.PASSWORD;
import static org.correomqtt.core.model.ConnectionPasswordType.SSL_KEYSTORE_PASSWORD;

//TODO check invalid configs

@SingletonBean
public class SettingsManager extends BaseUserFileProvider {

    private static final Logger LOGGER = LoggerFactory.getLogger(SettingsManager.class);

    private static final String CONFIG_FILE_NAME = "config.json";
    private final SecretStoreProvider secretStoreProvider;
    private final Set<Runnable> connectionChangeListeners = new HashSet<>();
    private String activeThemeName;
    private ConfigDTO configDTO;

    @Inject
    public SettingsManager(SoyEvents soyEvents,
                           SecretStoreProvider secretStoreProvider) {
        super(soyEvents);
        this.secretStoreProvider = secretStoreProvider;

        try {
            prepareFile(CONFIG_FILE_NAME);
        } catch (InvalidPathException | SecurityException | UnsupportedOperationException | IOException e) {
            LOGGER.error("Error writing config file {}. ", CONFIG_FILE_NAME, e);
            soyEvents.fire(new UnaccessibleConfigFileEvent(e));
        }

        try {
            configDTO = new ObjectMapper().readValue(getFile(), ConfigDTO.class);
        } catch (IOException e) {
            LOGGER.error("Exception parsing config file {}.", CONFIG_FILE_NAME, e);
            soyEvents.fire(new InvalidConfigFileEvent(e));
        }

    }

    public void addConnectionChangeListener(Runnable handler) {
        connectionChangeListeners.add(handler);
    }

    public void removeConnectionChangeListener(Runnable handler) {
        connectionChangeListeners.remove(handler);
    }

    public MessageListViewConfig produceSubscribeListViewConfig(String connectionId) {
        return configDTO.getConnections()
                .stream()
                .filter(c -> c.getId().equals(connectionId))
                .findFirst()
                .orElseThrow(CorreoMqttConfigurationMissingException::new)
                .produceSubscribeListViewConfig();
    }

    public MessageListViewConfig producePublishListViewConfig(String connectionId) {
        return configDTO.getConnections()
                .stream()
                .filter(c -> c.getId().equals(connectionId))
                .findFirst()
                .orElseThrow(CorreoMqttConfigurationMissingException::new)
                .producePublishListViewConfig();
    }

    public SettingsDTO getSettings() {
        return configDTO.getSettings();
    }

    public ThemeSettingsDTO getThemeSettings() {
        return configDTO.getThemesSettings();
    }

    public void saveSettings() {
        saveDTO();

        soyEvents.fire(new SettingsUpdatedEvent(false));
    }

    private void saveDTO() {

        try {
            new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(getFile(), configDTO);
        } catch (IOException e) {
            soyEvents.fire(new ConfigSaveFailedEvent(e));
        }
    }

    public String getActiveTheme() {
        if (activeThemeName == null) {
            if (configDTO.getThemesSettings().getNextTheme() != null) {
                configDTO.getThemesSettings().setActiveTheme(configDTO.getThemesSettings().getNextTheme());
                configDTO.getThemesSettings().setNextTheme(null);
                saveDTO();
            }
            activeThemeName = configDTO.getThemesSettings().getActiveTheme().getName();
        }
        return activeThemeName;
    }

    public void wipeSecretData(String masterPassword) throws EncryptionRecoverableException {
        List<ConnectionConfigDTO> connections = this.getConnectionConfigs();
        connections.forEach(c -> {
            c.setPassword(null);
            c.setAuthPassword(null);
            c.setSslKeystorePassword(null);
        });
        saveConnections(connections, masterPassword);
    }

    public List<ConnectionConfigDTO> getConnectionConfigs() {
        return configDTO.getConnections();
    }

    public void saveConnections(List<ConnectionConfigDTO> connections, String masterPassword) throws EncryptionRecoverableException {
        configDTO.setConnections(connections);
        saveDTO();

        for (ConnectionConfigDTO c : connections) {
            secretStoreProvider.setPassword(masterPassword, c, PASSWORD, c.getPassword());
            secretStoreProvider.setPassword(masterPassword, c, AUTH_PASSWORD, c.getAuthPassword());
            secretStoreProvider.setPassword(masterPassword, c, SSL_KEYSTORE_PASSWORD, c.getSslKeystorePassword());
        }
        secretStoreProvider.encryptAndSavePasswords(masterPassword);

        connectionChangeListeners.forEach(Runnable::run);
        soyEvents.fire(new ConnectionsUpdatedEvent());
    }

    public void initializePasswords(String masterPassword) throws EncryptionRecoverableException {
        secretStoreProvider.migratePasswordEncryption(masterPassword);

        List<ConnectionConfigDTO> connections = this.getConnectionConfigs();
        for (ConnectionConfigDTO c : connections) {
            c.setPassword(secretStoreProvider.getPassword(masterPassword, c, PASSWORD));
            c.setAuthPassword(secretStoreProvider.getPassword(masterPassword, c, AUTH_PASSWORD));
            c.setSslKeystorePassword(secretStoreProvider.getPassword(masterPassword, c, SSL_KEYSTORE_PASSWORD));
        }
        saveConnections(connections, masterPassword);
    }
}
