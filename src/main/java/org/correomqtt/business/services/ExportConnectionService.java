package org.correomqtt.business.services;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.correomqtt.business.dispatcher.ExportConnectionDispatcher;
import org.correomqtt.business.model.ConnectionConfigDTO;
import org.correomqtt.business.model.ExportConnectionView;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.File;
import java.io.IOException;
import java.util.List;

public class ExportConnectionService extends BaseService {

    private static final Logger LOGGER = LoggerFactory.getLogger(ExportConnectionService.class);

    private final File file;
    private final List<ConnectionConfigDTO> connectionConfigDTOS;

    public ExportConnectionService(String connectionId, File file, List<ConnectionConfigDTO> connectionConfigDTOS) {
        super(connectionId);
        this.file = file;
        this.connectionConfigDTOS = connectionConfigDTOS;
    }

    public void exportConnection() {
        try {
            new ObjectMapper().writerWithView(ExportConnectionView.class).writeValue(file, connectionConfigDTOS);
        } catch (IOException e) {
            ExportConnectionDispatcher.getInstance().onExportFailed(file,e);
        }
    }


    @Override
    public void onSucceeded() {
        LOGGER.info("Exporting connections to file {} succeeded", file.getAbsolutePath());
        ExportConnectionDispatcher.getInstance().onExportSucceeded();
    }

    @Override
    public void onCancelled() {
        LOGGER.info("Exporting connections to file {} cancelled", file.getAbsolutePath());
        ExportConnectionDispatcher.getInstance().onExportCancelled(file);
    }

    @Override
    public void onFailed(Throwable exception) {
        LOGGER.info("Exporting connections to file {} failed", file.getAbsolutePath());
        ExportConnectionDispatcher.getInstance().onExportFailed(file, exception);
    }

    @Override
    public void onRunning() {
        LOGGER.info("Exporting connections to file {} running", file.getAbsolutePath());
        ExportConnectionDispatcher.getInstance().onExportRunning();
    }

    @Override
    public void onScheduled() {
        LOGGER.info("Exporting connections to file {} scheduled", file.getAbsolutePath());
        ExportConnectionDispatcher.getInstance().onExportScheduled();
    }

}
