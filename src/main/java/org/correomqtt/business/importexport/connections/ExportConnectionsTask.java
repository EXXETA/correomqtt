package org.correomqtt.business.importexport.connections;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.correomqtt.business.concurrent.Task;
import org.correomqtt.business.encryption.Encryptor;
import org.correomqtt.business.encryption.EncryptorAesGcm;
import org.correomqtt.business.exception.CorreoMqttExecutionException;
import org.correomqtt.business.model.ConnectionConfigDTO;
import org.correomqtt.business.model.ConnectionConfigDTOMixin;
import org.correomqtt.business.model.ConnectionExportDTO;
import org.correomqtt.business.fileprovider.EncryptionRecoverableException;

import java.io.File;
import java.io.IOException;
import java.util.List;

public class ExportConnectionsTask extends Task<Integer, ExportConnectionsTask.Error> {

    public enum Error {
        EMPTY_COLLECTION_LIST,
        EMPTY_PASSWORD,
        FILE_IS_NULL,
        MISSING_FILE_EXTENSION
    }

    private final File file;
    private final List<ConnectionConfigDTO> connectionList;
    private final String password;

    public ExportConnectionsTask(File file, List<ConnectionConfigDTO> connectionList, String password) {
        this.file = file;
        this.connectionList = connectionList;
        this.password = password;
    }

    @Override
    protected Integer execute() {

        if (file == null) {
            throw createExpectedException(Error.FILE_IS_NULL);
        }

        if (connectionList == null || connectionList.isEmpty()) {
            throw createExpectedException(Error.EMPTY_COLLECTION_LIST);
        }

        if (file.getName().length() < 5 || !file.getName().endsWith(".cqc")) {
            throw createExpectedException(Error.MISSING_FILE_EXTENSION);
        }

        // TODO: positive way selecting stuff to export
        connectionList.forEach(connectionConfigDTO -> connectionConfigDTO.setConnectionUISettings(null));

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

        return connectionList.size();
    }
}
