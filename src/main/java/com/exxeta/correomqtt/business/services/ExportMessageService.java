package com.exxeta.correomqtt.business.services;

import com.exxeta.correomqtt.business.dispatcher.ExportMessageDispatcher;
import com.exxeta.correomqtt.business.exception.CorreoMqttExportMessageException;
import com.exxeta.correomqtt.business.model.MessageDTO;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.File;
import java.io.IOException;

public class ExportMessageService extends BaseService {

    private static final Logger LOGGER = LoggerFactory.getLogger(ExportMessageService.class);

    private final File file;
    private final MessageDTO messageDTO;

    public ExportMessageService(String connectionId, File file, MessageDTO messageDTO) {
        super(connectionId);
        this.file = file;
        this.messageDTO = messageDTO;
    }

    public void exportMessage() {
        ExportMessageDispatcher.getInstance().onExportStarted(connectionId, file, messageDTO);
        LOGGER.info(getConnectionMarker(), "Start exporting message {} to file {}.", messageDTO.getMessageId(), file.getAbsolutePath());
        try {
            new ObjectMapper().writeValue(file, messageDTO);
        } catch (IOException e) {
            throw new CorreoMqttExportMessageException(e);
        }
    }

    @Override
    public void onSucceeded() {
        LOGGER.info(getConnectionMarker(), "Exporting message {} to file {} succeeded.", messageDTO.getMessageId(), file.getAbsolutePath());
        ExportMessageDispatcher.getInstance().onExportSucceeded(connectionId);
    }

    @Override
    public void onCancelled() {
        LOGGER.info(getConnectionMarker(), "Exporting message {} to file {} cancelled.", messageDTO.getMessageId(), file.getAbsolutePath());
        ExportMessageDispatcher.getInstance().onExportCancelled(connectionId, file, messageDTO);
    }

    @Override
    public void onFailed(Throwable exception) {
        LOGGER.warn(getConnectionMarker(), "Exporting message {} to file {} failed.", messageDTO.getMessageId(), file.getAbsolutePath(), exception);
        ExportMessageDispatcher.getInstance().onExportFailed(connectionId, file, messageDTO, exception);
    }

    @Override
    public void onRunning() {
        LOGGER.info(getConnectionMarker(), "Exporting message {} to file {} running.", messageDTO.getMessageId(), file.getAbsolutePath());
        ExportMessageDispatcher.getInstance().onExportRunning(connectionId);
    }

    @Override
    public void onScheduled() {
        LOGGER.info(getConnectionMarker(), "Exporting message {} to file {} scheduled.", messageDTO.getMessageId(), file.getAbsolutePath());
        ExportMessageDispatcher.getInstance().onExportScheduled(connectionId);
    }
}

