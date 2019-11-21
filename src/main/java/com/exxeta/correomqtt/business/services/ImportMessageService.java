package com.exxeta.correomqtt.business.services;

import com.exxeta.correomqtt.business.dispatcher.ImportMessageDispatcher;
import com.exxeta.correomqtt.business.exception.CorreoMqttExportMessageException;
import com.exxeta.correomqtt.business.model.MessageDTO;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.File;
import java.io.IOException;

public class ImportMessageService extends BaseService {

    private static final Logger LOGGER = LoggerFactory.getLogger(ImportMessageService.class);

    private final File file;
    private MessageDTO messageDTO;

    public ImportMessageService(String connectionId, File file) {
        super(connectionId);
        this.file = file;
    }

    public void importMessage() {
        ImportMessageDispatcher.getInstance().onImportStarted(connectionId, file);
        LOGGER.info(getConnectionMarker(), "Start importing message from file {}.", file.getAbsolutePath());
        try {
            messageDTO = new ObjectMapper().readValue(file, MessageDTO.class);
        } catch (IOException e) {
            throw new CorreoMqttExportMessageException(e);
        }
    }

    @Override
    public void onSucceeded() {
        LOGGER.info(getConnectionMarker(), "Importing message from file {} succeeded", file.getAbsolutePath());
        ImportMessageDispatcher.getInstance().onImportSucceeded(connectionId, messageDTO);
    }

    @Override
    public void onCancelled() {
        LOGGER.info(getConnectionMarker(), "Importing message from file {} cancelled", file.getAbsolutePath());
        ImportMessageDispatcher.getInstance().onImportCancelled(connectionId, file);
    }

    @Override
    public void onFailed(Throwable exception) {
        LOGGER.info(getConnectionMarker(), "Importing message from file {} failed.", file.getAbsolutePath(), exception);
        ImportMessageDispatcher.getInstance().onImportFailed(connectionId, file, exception);
    }

    @Override
    public void onRunning() {
        LOGGER.info(getConnectionMarker(), "Importing message from file {} running.", file.getAbsolutePath());
        ImportMessageDispatcher.getInstance().onImportRunning(connectionId);
    }

    @Override
    public void onScheduled() {
        LOGGER.info(getConnectionMarker(), "Importing message from file {} scheduled.", file.getAbsolutePath());
        ImportMessageDispatcher.getInstance().onImportScheduled(connectionId);
    }
}

