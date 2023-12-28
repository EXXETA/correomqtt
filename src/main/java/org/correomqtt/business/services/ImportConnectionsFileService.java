package org.correomqtt.business.services;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.correomqtt.business.dispatcher.ImportConnectionsFileDispatcher;
import org.correomqtt.business.exception.CorreoMqttExecutionException;
import org.correomqtt.business.model.ConnectionExportDTO;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.File;
import java.io.IOException;

public class ImportConnectionsFileService implements BusinessService {


    private static final Logger LOGGER = LoggerFactory.getLogger(ImportConnectionsFileService.class);

    private final File file;
    private ConnectionExportDTO connectionExportDTO;

    public ImportConnectionsFileService(File file) {
        this.file = file;
    }

    public void importConnections() {
        ImportConnectionsFileDispatcher.getInstance().onImportStarted(file);
        LOGGER.info("Start importing connections from file {}.", file.getAbsolutePath());
        try {
            connectionExportDTO= new ObjectMapper().readerFor(ConnectionExportDTO.class).readValue(file);
            LOGGER.info("Start importing connections from file {}.", file.getAbsolutePath());

        } catch (IOException e) {
            throw new CorreoMqttExecutionException(e);
        }
    }


    @Override
    public void onSucceeded() {
        LOGGER.info("Importing connections from file {} succeeded", file.getAbsolutePath());
        ImportConnectionsFileDispatcher.getInstance().onImportSucceeded(connectionExportDTO);
    }

    @Override
    public void onCancelled() {
        LOGGER.info("Importing connections from file {} cancelled", file.getAbsolutePath());
        ImportConnectionsFileDispatcher.getInstance().onImportCancelled(file);
    }

    @Override
    public void onFailed(Throwable exception) {
        LOGGER.info("Importing connections from file {} failed.", file.getAbsolutePath(), exception);
        ImportConnectionsFileDispatcher.getInstance().onImportFailed(file, exception);

    }

    @Override
    public void onRunning() {
        LOGGER.info("Importing connections from file {} running.", file.getAbsolutePath());
        ImportConnectionsFileDispatcher.getInstance().onImportRunning();

    }

    @Override
    public void onScheduled() {
        LOGGER.info("Importing connections from file {} scheduled.", file.getAbsolutePath());
        ImportConnectionsFileDispatcher.getInstance().onImportScheduled();

    }
}
