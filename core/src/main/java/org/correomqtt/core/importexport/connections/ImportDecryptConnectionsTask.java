package org.correomqtt.core.importexport.connections;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.correomqtt.core.concurrent.NoProgressTask;
import org.correomqtt.core.concurrent.TaskException;
import org.correomqtt.core.encryption.EncryptorAesGcm;
import org.correomqtt.core.fileprovider.EncryptionRecoverableException;
import org.correomqtt.core.model.ConnectionConfigDTO;

import java.util.List;

public class ImportDecryptConnectionsTask extends NoProgressTask<List<ConnectionConfigDTO>, ImportDecryptConnectionsTask.Error> {


    public enum Error {
        ENCRYPTION_TYPE_NOT_ALLOWED,
        FILE_CAN_NOT_BE_READ_OR_PARSED
    }

    private final String encryptedData;
    private final String encryptionType;
    private final String password;

    public ImportDecryptConnectionsTask(String encryptedData, String encryptionType, String password) {

        this.encryptedData = encryptedData;
        this.encryptionType = encryptionType;
        this.password = password;
    }

    @Override
    protected List<ConnectionConfigDTO> execute() throws TaskException, EncryptionRecoverableException, JsonProcessingException {
        EncryptorAesGcm encryptor = new EncryptorAesGcm(password);
        if (!encryptor.getEncryptionTranslation().equals(encryptionType)) {
            throw new TaskException(Error.ENCRYPTION_TYPE_NOT_ALLOWED);
        }

        String connectionsString = encryptor.decrypt(this.encryptedData);
        return new ObjectMapper().readerFor(new TypeReference<List<ConnectionConfigDTO>>() {
        }).readValue(connectionsString);

    }
}
