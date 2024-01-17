package org.correomqtt.business.fileprovider;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.correomqtt.business.eventbus.EventBus;
import org.correomqtt.business.exception.CorreoMqttConfigurationMissingException;
import org.correomqtt.business.model.ConfigDTO;
import org.correomqtt.business.model.ConnectionConfigDTO;
import org.correomqtt.business.model.MessageListViewConfig;
import org.correomqtt.business.model.SettingsDTO;
import org.correomqtt.business.model.ThemeSettingsDTO;
import org.correomqtt.business.utils.ConnectionHolder;
import org.correomqtt.gui.keyring.KeyringHandler;
import org.correomqtt.gui.theme.ThemeProvider;
import org.correomqtt.gui.theme.light_legacy.LightLegacyThemeProvider;
import org.correomqtt.plugin.manager.PluginManager;
import org.correomqtt.plugin.spi.ThemeProviderHook;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.File;
import java.io.IOException;
import java.nio.file.InvalidPathException;
import java.util.ArrayList;
import java.util.List;

import static org.correomqtt.business.model.ConnectionPasswordType.AUTH_PASSWORD;
import static org.correomqtt.business.model.ConnectionPasswordType.PASSWORD;
import static org.correomqtt.business.model.ConnectionPasswordType.SSL_KEYSTORE_PASSWORD;

//TODO check invalid configs

public class SettingsProvider extends BaseUserFileProvider {

    private static final Logger LOGGER = LoggerFactory.getLogger(SettingsProvider.class);

    private static final String CONFIG_FILE_NAME = "config.json";
    private static final String CSS_FILE_NAME = "style.css";

    private ThemeProvider activeThemeProvider;

    private static SettingsProvider instance = null;

    private ConfigDTO configDTO;

    private SettingsProvider() {

        try {
            prepareFile(CONFIG_FILE_NAME);
        } catch (InvalidPathException | SecurityException | UnsupportedOperationException | IOException e) {
            LOGGER.error("Error writing config file {}. ", CONFIG_FILE_NAME, e);
            EventBus.fire(new UnaccessibleConfigFileEvent(e));
        }

        try {
            configDTO = new ObjectMapper().readValue(getFile(), ConfigDTO.class);
        } catch (IOException e) {
            LOGGER.error("Exception parsing config file {}.", CONFIG_FILE_NAME, e);
            EventBus.fire(new InvalidConfigFileEvent(e));
        }

    }

    public static synchronized SettingsProvider getInstance() {
        if (instance == null) {
            instance = new SettingsProvider();
            return instance;
        } else {
            return instance;
        }
    }

    public ThemeProvider getActiveTheme() {
        if (activeThemeProvider == null) {
            if(configDTO.getThemesSettings().getNextTheme() != null) {
                configDTO.getThemesSettings().setActiveTheme(configDTO.getThemesSettings().getNextTheme());
                configDTO.getThemesSettings().setNextTheme(null);
                saveDTO();
            }
            String activeThemeName = configDTO.getThemesSettings().getActiveTheme().getName();
            ArrayList<ThemeProvider> themes = new ArrayList<>(PluginManager.getInstance().getExtensions(ThemeProviderHook.class));
            activeThemeProvider = themes.stream().filter(t -> t.getName().equals(activeThemeName)).findFirst().orElse(new LightLegacyThemeProvider());
        }
        return activeThemeProvider;
    }

    public List<ConnectionConfigDTO> getConnectionConfigs() {
        return configDTO.getConnections();
    }

    public MessageListViewConfig produceSubscribeListViewConfig(String connectionId){
        return configDTO.getConnections()
                .stream()
                .filter(c -> c.getId().equals(connectionId))
                .findFirst()
                .orElseThrow(CorreoMqttConfigurationMissingException::new)
                .produceSubscribeListViewConfig();
    }

    public MessageListViewConfig producePublishListViewConfig(String connectionId){
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

    public void saveSettings(boolean restartRequired) {
        saveDTO();
        saveToUserDirectory(CSS_FILE_NAME, getActiveTheme().getCss());
        EventBus.fire(new SettingsUpdatedEvent(restartRequired));
    }

    public void saveConnections(List<ConnectionConfigDTO> connections, String masterPassword) throws EncryptionRecoverableException {
        configDTO.setConnections(connections);
        saveDTO();

        SecretStoreProvider secretStoreProvider = SecretStoreProvider.getInstance();
        for(ConnectionConfigDTO c: connections){
            secretStoreProvider.setPassword(masterPassword, c, PASSWORD, c.getPassword());
            secretStoreProvider.setPassword(masterPassword,c, AUTH_PASSWORD,  c.getAuthPassword());
            secretStoreProvider.setPassword(masterPassword, c, SSL_KEYSTORE_PASSWORD, c.getSslKeystorePassword());
        }
        secretStoreProvider.encryptAndSavePasswords(masterPassword);

        ConnectionHolder.getInstance().refresh();
        EventBus.fire(new ConnectionsUpdatedEvent());
    }

    private void saveDTO() {

        try {
            new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(getFile(), configDTO);
        } catch (IOException e) {
            EventBus.fire(new ConfigSaveFailedEvent(e));
        }
    }

    public void wipeSecretData(String masterPassword) throws EncryptionRecoverableException {
        KeyringHandler.getInstance().wipe();
        List<ConnectionConfigDTO> connections = this.getConnectionConfigs();
        connections.forEach(c -> {
            c.setPassword(null);
            c.setAuthPassword(null);
            c.setSslKeystorePassword(null);
        });
        saveConnections(connections, masterPassword);
    }

    public String getCssPath() {
        File cssFile = new File(getTargetDirectoryPath() + File.separator + CSS_FILE_NAME);
        if (!cssFile.exists()) {
            saveToUserDirectory(CSS_FILE_NAME, getActiveTheme().getCss());
        }
        if (cssFile.exists()) {
            return cssFile.toURI().toString();
        } else {
            return null;
        }
    }

    public String getLogPath() {
        return getTargetDirectoryPath() + File.separator;
    }

    public String getIconModeCssClass() {
        return SettingsProvider.getInstance().getActiveTheme().getIconMode().toString();
    }

    public void initializePasswords(String masterPassword) throws EncryptionRecoverableException {
        SecretStoreProvider secretStoreProvider = SecretStoreProvider.getInstance();

        secretStoreProvider.migratePasswordEncryption(masterPassword);

        List<ConnectionConfigDTO> connections = this.getConnectionConfigs();
        for(ConnectionConfigDTO c: connections){
            c.setPassword(secretStoreProvider.getPassword(masterPassword,c,PASSWORD));
            c.setAuthPassword(secretStoreProvider.getPassword(masterPassword,c, AUTH_PASSWORD));
            c.setSslKeystorePassword(secretStoreProvider.getPassword(masterPassword,c, SSL_KEYSTORE_PASSWORD));
        }
        saveConnections(connections, masterPassword);
    }
}
