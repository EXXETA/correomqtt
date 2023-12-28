package org.correomqtt.business.services;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.correomqtt.business.dispatcher.ExportConnectionDispatcher;
import org.correomqtt.business.encryption.Encryptor;
import org.correomqtt.business.encryption.EncryptorAesGcm;
import org.correomqtt.business.exception.CorreoMqttExecutionException;
import org.correomqtt.business.model.ConnectionConfigDTO;
import org.correomqtt.business.model.ConnectionConfigDTOMixin;
import org.correomqtt.business.model.ConnectionExportDTO;
import org.correomqtt.business.provider.EncryptionRecoverableException;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.File;
import java.io.IOException;
import java.util.List;

public class ExportConnectionsService implements BusinessService {

    private static final Logger LOGGER = LoggerFactory.getLogger(ExportConnectionsService.class);

    private final File file;
    private final List<ConnectionConfigDTO> connectionList;
    private final String password;

    public ExportConnectionsService(File file, List<ConnectionConfigDTO> connectionList, String password) {
        this.file = file;
        this.connectionList = connectionList;
        this.password = password;
    }

    public void exportConnections() {

        // TODO: positive way selecting stuff to export
        if (connectionList != null) {
            connectionList.forEach(connectionConfigDTO -> connectionConfigDTO.setConnectionUISettings(null));
        }

        ConnectionExportDTO connectionExportDTO;

        try {
            if (password == null) {
                connectionExportDTO = new ConnectionExportDTO(connectionList);
            } else {
                // TODO check usage of mixin? What is it here?
                String connectionsJSON = new ObjectMapper().addMixIn(ConnectionConfigDTO.class, ConnectionConfigDTOMixin.class).writeValueAsString(connectionList);
                Encryptor encryptor = new EncryptorAesGcm(password);
                String encryptedData = new EncryptorAesGcm(password).encrypt(connectionsJSON);
                connectionExportDTO = new ConnectionExportDTO(encryptor.getEncryptionTranslation(), encryptedData);
            }
            new ObjectMapper().writeValue(file, connectionExportDTO);
        } catch (EncryptionRecoverableException | IOException e) {
            throw new CorreoMqttExecutionException(e);
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
