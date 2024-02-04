package org.correomqtt.core.fileprovider;

import com.fasterxml.jackson.databind.ObjectMapper;
import lombok.Getter;
import org.correomqtt.core.eventbus.EventBus;
import org.correomqtt.core.settings.SettingsManager;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.InvalidPathException;

@Getter
abstract class BasePersistHistoryProvider<D> extends BaseUserFileProvider {

    private static final Logger LOGGER = LoggerFactory.getLogger(BasePersistHistoryProvider.class);

    abstract String getHistoryFileName();

    abstract Class<D> getDTOClass();

    abstract void setDTO(String id, D readValue);

    private final SettingsManager settings;
    private final String connectionId;

    BasePersistHistoryProvider(SettingsManager settings,
                               EventBus eventBus,
                               String id) {
        super(eventBus);
        this.settings = settings;
        connectionId = id;
        String historyFileName = getHistoryFileName();
        try {
            prepareFile(id, historyFileName);
        } catch (UnsupportedOperationException | InvalidPathException | SecurityException | IOException e) {
            LOGGER.error("Error reading " + historyFileName, e);
            readingError(e);
        }
        try {
            setDTO(id, new ObjectMapper().readValue(getFile(), getDTOClass()));
        } catch (IOException e) {
            LOGGER.error("Error reading " + historyFileName, e);
            readingError(e);
        }
    }

    protected abstract void readingError(Exception e);

    protected void removeFileIfConnectionDeleted() {
        settings.getConnectionConfigs().stream()
                .filter(c -> c.getId().equals(getConnectionId()))
                .findFirst()
                .ifPresentOrElse(c -> {
                }, () -> {
                    try {
                        Files.delete(getFile().toPath());
                        LOGGER.info("{} deleted successfully", getFile());
                    } catch (IOException e) {
                        LOGGER.info("Failed to delete {}", getFile(), e);
                    }
                });
    }
}


