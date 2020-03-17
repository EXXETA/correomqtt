package org.correomqtt.business.services;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;
import java.nio.file.InvalidPathException;

abstract class BasePersistHistoryService<D> extends BaseUserFileService {

    private static final Logger LOGGER = LoggerFactory.getLogger(BasePersistHistoryService.class);

    abstract String getHistoryFileName();

    abstract Class<D> getDTOClass();

    public String getConnectionId() {
        return connectionId;
    };

    abstract void setDTO(String id, D readValue);
    private String connectionId;

    BasePersistHistoryService(String id) {
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
        ConfigService.getInstance().getConnectionConfigs().stream()
                .filter(c -> c.getId().equals(getConnectionId()))
                .findFirst()
                .ifPresentOrElse(c -> {
                }, () -> {
                    if (getFile().delete()) {
                        LOGGER.info(getFile() + " deleted successfully");
                    } else {
                        LOGGER.info("Failed to delete " + getFile());
                    }
                });
    }
}


