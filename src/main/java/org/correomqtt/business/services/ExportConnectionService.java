package org.correomqtt.business.services;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.correomqtt.business.dispatcher.ExportConnectionDispatcher;
import org.correomqtt.business.model.ConnectionExportDTO;
import org.correomqtt.business.model.ExportConnectionView;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.File;
import java.io.IOException;

public class ExportConnectionService extends BaseService {

    private static final Logger LOGGER = LoggerFactory.getLogger(ExportConnectionService.class);

    private final File file;
    private final ConnectionExportDTO connectionExportDTO;

    public ExportConnectionService(String connectionId, File file, ConnectionExportDTO connectionExportDTO) {
        super(connectionId);
        this.file = file;
        this.connectionExportDTO = connectionExportDTO;
    }

    public void exportConnection() {
        try {
                new ObjectMapper().writeValue(file, connectionExportDTO);
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
