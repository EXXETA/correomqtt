package org.correomqtt.business.services;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.correomqtt.business.dispatcher.ExportMessageDispatcher;
import org.correomqtt.business.exception.CorreoMqttExportMessageException;
import org.correomqtt.business.model.ConnectionConfigDTO;
import org.correomqtt.business.model.ExportConnectionView;
import org.correomqtt.business.model.MessageDTO;
import org.correomqtt.gui.model.ConnectionPropertiesDTO;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.File;
import java.io.IOException;
import java.util.ArrayList;
import java.util.List;

public class ExportConnectionService extends BaseService{

    private static final Logger LOGGER = LoggerFactory.getLogger(ExportMessageService.class);

    private final File file;
    private final List<ConnectionConfigDTO> connectionConfigDTOS;

    public ExportConnectionService(String connectionId, File file, List<ConnectionConfigDTO> connectionConfigDTOS) {
        super(connectionId);
        this.file = file;
        this.connectionConfigDTOS = connectionConfigDTOS;
    }

    public void exportConnection() {
        LOGGER.info( "Start exporting connections to file {}.", file.getAbsolutePath());
        try {
            new ObjectMapper().writerWithView(ExportConnectionView.class).writeValue(file,connectionConfigDTOS);
        } catch (IOException e) {
            throw new CorreoMqttExportMessageException(e);
        }
    }



    public void onSucceeded() {

    }


    public void onCancelled() {

    }


    public void onFailed(Throwable exception) {

    }

    public void onRunning() {

    }


    public void onScheduled() {

    }
}
