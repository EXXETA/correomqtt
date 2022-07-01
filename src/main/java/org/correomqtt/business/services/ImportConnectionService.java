package org.correomqtt.business.services;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.correomqtt.business.dispatcher.ImportConnectionDispatcher;
import org.correomqtt.business.dispatcher.ImportMessageDispatcher;
import org.correomqtt.business.exception.CorreoMqttExportMessageException;
import org.correomqtt.business.model.ConnectionConfigDTO;
import org.correomqtt.business.model.ExportConnectionView;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.File;
import java.io.IOException;
import java.util.ArrayList;
import java.util.List;

public class ImportConnectionService extends BaseService{



    private static final Logger LOGGER = LoggerFactory.getLogger(ExportMessageService.class);

    private final File file;
    private List<ConnectionConfigDTO> connectionConfigDTOS;

    public ImportConnectionService(String connectionId, File file) {
        super(connectionId);
        this.file = file;
        this.connectionConfigDTOS = new ArrayList<>();
    }

    public void importConnection() {
        ImportConnectionDispatcher.getInstance().onImportStarted(connectionId,file);
        LOGGER.info( "Start exporting connections to file {}.", file.getAbsolutePath());
        try {
            connectionConfigDTOS = new ObjectMapper().readerWithView(ExportConnectionView.class).readValue(file);
        } catch (IOException e) {
            throw new CorreoMqttExportMessageException(e);
        }
    }



    @Override
    public void onSucceeded() {
        LOGGER.info(getConnectionMarker(), "Importing connections from file {} succeeded", file.getAbsolutePath());
        ImportConnectionDispatcher.getInstance().onImportSucceeded(connectionId,connectionConfigDTOS);
    }

    @Override
    public void onCancelled() {
        LOGGER.info(getConnectionMarker(), "Importing connections from file {} cancelled", file.getAbsolutePath());
        ImportConnectionDispatcher.getInstance().onImportCancelled(connectionId,file);
    }

    @Override
    public void onFailed(Throwable exception) {
        LOGGER.info(getConnectionMarker(), "Importing connections from file {} failed.", file.getAbsolutePath(), exception);
        ImportConnectionDispatcher.getInstance().onImportFailed(connectionId, file, exception);

    }

    @Override
    public void onRunning() {
        LOGGER.info(getConnectionMarker(), "Importing connections from file {} running.", file.getAbsolutePath());
        ImportConnectionDispatcher.getInstance().onImportRunning(connectionId);

    }

    @Override
    public void onScheduled() {
        LOGGER.info(getConnectionMarker(), "Importing connections from file {} scheduled.", file.getAbsolutePath());
        ImportConnectionDispatcher.getInstance().onImportScheduled(connectionId);

    }
}
