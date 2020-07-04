package org.correomqtt.business.provider;

import com.fasterxml.jackson.core.JsonGenerationException;
import com.fasterxml.jackson.databind.JsonMappingException;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.correomqtt.business.dispatcher.ConfigDispatcher;
import org.correomqtt.business.keyring.Keyring;
import org.correomqtt.business.model.ConfigDTO;
import org.correomqtt.business.model.ConnectionConfigDTO;
import org.correomqtt.business.model.SettingsDTO;
import org.correomqtt.business.model.ThemeSettingsDTO;
import org.correomqtt.business.utils.ConnectionHolder;
import org.correomqtt.gui.keyring.KeyringHandler;
import org.correomqtt.gui.theme.ThemeProvider;
import org.correomqtt.gui.theme.light.LightThemeProvider;
import org.correomqtt.plugin.manager.PluginManager;
import org.correomqtt.plugin.spi.ThemeProviderHook;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.File;
import java.io.FileNotFoundException;
import java.io.IOException;
import java.nio.file.AccessDeniedException;
import java.nio.file.DirectoryNotEmptyException;
import java.nio.file.FileAlreadyExistsException;
import java.nio.file.InvalidPathException;
import java.util.ArrayList;
import java.util.List;

import static org.correomqtt.business.model.ConnectionPasswordType.*;

//TODO check invalid configs

public class SettingsProvider extends BaseUserFileProvider {

    private static final Logger LOGGER = LoggerFactory.getLogger(SettingsProvider.class);

    private static final String CONFIG_FILE_NAME = "config.json";
    private static final String CSS_FILE_NAME = "style.css";
    private static final String EX_MSG_PREPARE_CONFIG = "Exception preparing config file.";
    private static final String EX_MSG_WRITE_CONFIG = "Exception writing config file.";

    private ThemeProvider activeThemeProvider;

    private static SettingsProvider instance = null;

    private ConfigDTO configDTO;

    private SettingsProvider() {

        try {
            prepareFile(CONFIG_FILE_NAME);
        } catch (InvalidPathException e) {
            LOGGER.error(EX_MSG_PREPARE_CONFIG, e);
            ConfigDispatcher.getInstance().onInvalidPath();
        } catch (FileAlreadyExistsException e) {
            LOGGER.error(EX_MSG_PREPARE_CONFIG, e);
            ConfigDispatcher.getInstance().onFileAlreadyExists();
        } catch (DirectoryNotEmptyException e) {
            LOGGER.error(EX_MSG_PREPARE_CONFIG, e);
            ConfigDispatcher.getInstance().onConfigDirectoryEmpty();
        } catch (SecurityException | AccessDeniedException e) {
            LOGGER.error(EX_MSG_PREPARE_CONFIG, e);
            ConfigDispatcher.getInstance().onConfigDirectoryNotAccessible();
        } catch (UnsupportedOperationException | IOException e) {
            LOGGER.error(EX_MSG_PREPARE_CONFIG, e);
            ConfigDispatcher.getInstance().onConfigPrepareFailure();
        }

        try {
            configDTO = new ObjectMapper().readValue(getFile(), ConfigDTO.class);
        } catch (IOException e) {
            LOGGER.error("Exception parsing config file.", e);
            ConfigDispatcher.getInstance().onInvalidJsonFormat();
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

    private ThemeProvider getActiveTheme() {
        if (activeThemeProvider == null) {
            String activeThemeName = configDTO.getThemesSettings().getActiveTheme().getName();
            ArrayList<ThemeProvider> themes = new ArrayList<>(PluginManager.getInstance().getExtensions(ThemeProviderHook.class));
            activeThemeProvider = themes.stream().filter(t -> t.getName().equals(activeThemeName)).findFirst().orElse(new LightThemeProvider());
        }
        return activeThemeProvider;
    }

    public List<ConnectionConfigDTO> getConnectionConfigs() {
        return configDTO.getConnections();
    }

    public SettingsDTO getSettings() {
        return configDTO.getSettings();
    }

    public ThemeSettingsDTO getThemeSettings() {
        return configDTO.getThemesSettings();
    }

    public void saveSettings() {
        this.activeThemeProvider = null;
        saveDTO();
        saveToUserDirectory(CSS_FILE_NAME, getActiveTheme().getCss());
        ConfigDispatcher.getInstance().onSettingsUpdated();
    }

    public void saveConnections(List<ConnectionConfigDTO> connections, String masterPassword) throws PasswordRecoverableException {
        configDTO.setConnections(connections);

        SecretStoreProvider secretStoreProvider = SecretStoreProvider.getInstance();
        for(ConnectionConfigDTO c: connections){
            secretStoreProvider.setPassword(masterPassword, c, PASSWORD, c.getPassword());
            secretStoreProvider.setPassword(masterPassword,c, AUTH_PASSWORD,  c.getAuthPassword());
            secretStoreProvider.setPassword(masterPassword, c, SSL_KEYSTORE_PASSWORD, c.getSslKeystorePassword());
        };
        secretStoreProvider.encryptAndSavePasswords(masterPassword);

        ConnectionHolder.getInstance().refresh();
        ConfigDispatcher.getInstance().onConnectionsUpdated();
    }

    private void saveDTO() {

        try {
            new ObjectMapper().writeValue(getFile(), configDTO);
        } catch (FileNotFoundException e) {
            LOGGER.error(EX_MSG_WRITE_CONFIG, e);
            ConfigDispatcher.getInstance().onConfigDirectoryEmpty();
        } catch (JsonGenerationException | JsonMappingException e) {
            LOGGER.error(EX_MSG_WRITE_CONFIG, e);
            ConfigDispatcher.getInstance().onInvalidJsonFormat();
        } catch (IOException e) {
            LOGGER.error(EX_MSG_WRITE_CONFIG, e);
            ConfigDispatcher.getInstance().onSavingFailed();
        }
    }

    public void wipeSecretData(String masterPassword) throws PasswordRecoverableException {
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
        return configDTO.getThemesSettings().getActiveTheme().getIconMode().toString();
    }

    public void initializePasswords(String masterPassword) throws PasswordRecoverableException {
        SecretStoreProvider secretStoreProvider = SecretStoreProvider.getInstance();
        List<ConnectionConfigDTO> connections = this.getConnectionConfigs();
        for(ConnectionConfigDTO c: connections){
            c.setPassword(secretStoreProvider.getPassword(masterPassword,c,PASSWORD));
            c.setAuthPassword(secretStoreProvider.getPassword(masterPassword,c, AUTH_PASSWORD));
            c.setSslKeystorePassword(secretStoreProvider.getPassword(masterPassword,c, SSL_KEYSTORE_PASSWORD));
        };
        saveConnections(connections, masterPassword);
    }
}
