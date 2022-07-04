package org.correomqtt.business.services;

import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.correomqtt.business.dispatcher.ImportConnectionDispatcher;
import org.correomqtt.business.exception.CorreoMqttExportMessageException;
import org.correomqtt.business.model.ConnectionConfigDTO;
import org.correomqtt.business.model.ExportConnectionView;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.File;
import java.io.IOException;
import java.util.ArrayList;
import java.util.List;

public class ImportConnectionService extends BaseService {


    private static final Logger LOGGER = LoggerFactory.getLogger(ImportConnectionService.class);

    private final File file;
    private List<ConnectionConfigDTO> connectionConfigDTOS;

    public ImportConnectionService(File file) {
        super(null);
        this.file = file;
        this.connectionConfigDTOS = new ArrayList<>();
    }

    public void importConnection() {
        ImportConnectionDispatcher.getInstance().onImportStarted(file);
        LOGGER.info("Start importing connections from file {}.", file.getAbsolutePath());
        try {
            connectionConfigDTOS = new ObjectMapper().readerWithView(ExportConnectionView.class).forType(new TypeReference<List<ConnectionConfigDTO>>() {
            }).readValue(file);
            LOGGER.info("Start importing connections from file {}.", file.getAbsolutePath());

        } catch (IOException e) {
            LOGGER.error("Importing Connections failed");
            ImportConnectionDispatcher.getInstance().onImportFailed(file,e);
        }
    }


    @Override
    public void onSucceeded() {
        LOGGER.info("Importing connections from file {} succeeded", file.getAbsolutePath());
        ImportConnectionDispatcher.getInstance().onImportSucceeded(connectionConfigDTOS);
    }

    @Override
    public void onCancelled() {
        LOGGER.info("Importing connections from file {} cancelled", file.getAbsolutePath());
        ImportConnectionDispatcher.getInstance().onImportCancelled(file);
    }

    @Override
    public void onFailed(Throwable exception) {
        LOGGER.info("Importing connections from file {} failed.", file.getAbsolutePath(), exception);
        ImportConnectionDispatcher.getInstance().onImportFailed(file, exception);

    }

    @Override
    public void onRunning() {
        LOGGER.info("Importing connections from file {} running.", file.getAbsolutePath());
        ImportConnectionDispatcher.getInstance().onImportRunning();

    }

    @Override
    public void onScheduled() {
        LOGGER.info("Importing connections from file {} scheduled.", file.getAbsolutePath());
        ImportConnectionDispatcher.getInstance().onImportScheduled();

    }
}
