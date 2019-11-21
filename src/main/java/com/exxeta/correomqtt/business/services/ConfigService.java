package com.exxeta.correomqtt.business.services;

import com.exxeta.correomqtt.business.dispatcher.ConfigDispatcher;
import com.exxeta.correomqtt.business.model.ConfigDTO;
import com.exxeta.correomqtt.business.model.ConnectionConfigDTO;
import com.exxeta.correomqtt.business.model.SettingsDTO;
import com.exxeta.correomqtt.business.model.ThemeSettingsDTO;
import com.exxeta.correomqtt.business.utils.ConnectionHolder;
import com.fasterxml.jackson.core.JsonGenerationException;
import com.fasterxml.jackson.databind.JsonMappingException;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.File;
import java.io.FileNotFoundException;
import java.io.IOException;
import java.nio.file.AccessDeniedException;
import java.nio.file.DirectoryNotEmptyException;
import java.nio.file.FileAlreadyExistsException;
import java.nio.file.InvalidPathException;
import java.util.List;

//TODO check invalid configs

public class ConfigService extends BaseUserFileService {

    private static final Logger LOGGER = LoggerFactory.getLogger(ConfigService.class);

    private static final String CONFIG_FILE_NAME = "config.json";
    public static final String LIGHT_CSS = "light.css";
    public static final String DARK_CSS = "dark.css";
    private static String CSS_FILE_NAME = null; //TODO move to gui package
    private static final String EX_MSG_PREPARE_CONFIG = "Exception preparing config file.";
    private static final String EX_MSG_WRITE_CONFIG = "Exception writing config file.";

    private static ConfigService connectionInstance = null;

    private ConfigDTO configDTO;

    private ConfigService() {

        try {
            prepareFile(LIGHT_CSS);
            prepareFile(DARK_CSS);
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

    public static synchronized ConfigService getInstance() {
        if (connectionInstance == null) {
            connectionInstance = new ConfigService();
            return connectionInstance;
        } else {
            return connectionInstance;
        }
    }

    public List<ConnectionConfigDTO> getConnectionConfigs() {
        return configDTO.getConnections();
    }

    public ThemeSettingsDTO getThemeSettings() {
        return configDTO.getThemesSettings();
    }

    public SettingsDTO getSettings() {
        return configDTO.getSettings();
    }

    public void saveSettings() {
        saveDTO();
        ConfigDispatcher.getInstance().onSettingsUpdated();
    }

    public void saveThemeSettings() {
        saveDTO();
    }

    public void saveConnections(List<ConnectionConfigDTO> connections) {
        configDTO.setConnections(connections);
        saveDTO();
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

    public String getCssPath() {
        return getCssPath(true);
    }

    public String getCssPath(boolean notifyFail) {
        File cssFile = new File(getTargetDirectoryPath() + File.separator + CSS_FILE_NAME);
        if (cssFile.exists()) {
            return cssFile.toURI().toString();
        } else {
            return null;
        }
    }

    public void setCssFileName() {
        if (configDTO.getThemesSettings().getActiveTheme() == null) {
            configDTO.getThemesSettings().getThemes().stream()
                    .findFirst()
                    .ifPresent(t ->  {
                        CSS_FILE_NAME = t.getFile();
                        configDTO.getThemesSettings().setActiveTheme(t);
                        ConfigService.getInstance().saveThemeSettings();
                    });
        } else {
            configDTO.getThemesSettings().getThemes().stream()
                    .filter(t -> t.getName().equals(configDTO.getThemesSettings().getActiveTheme().getName()))
                    .findFirst()
                    .ifPresent(t -> CSS_FILE_NAME = t.getFile());
        }
    }

    public String getLogPath() {
        return getTargetDirectoryPath() + File.separator;
    }
}
