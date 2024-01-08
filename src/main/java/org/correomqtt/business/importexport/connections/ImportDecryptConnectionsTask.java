package org.correomqtt.business.importexport.connections;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.correomqtt.business.concurrent.NoProgressTask;
import org.correomqtt.business.encryption.EncryptorAesGcm;
import org.correomqtt.business.model.ConnectionConfigDTO;
import org.correomqtt.business.fileprovider.EncryptionRecoverableException;

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
    protected List<ConnectionConfigDTO> execute() {
        EncryptorAesGcm encryptor = new EncryptorAesGcm(password);
        if (!encryptor.getEncryptionTranslation().equals(encryptionType)) {
            throw createExpectedException(Error.ENCRYPTION_TYPE_NOT_ALLOWED);
        }

        try {
            String connectionsString = encryptor.decrypt(this.encryptedData);
            return new ObjectMapper().readerFor(new TypeReference<List<ConnectionConfigDTO>>() {
            }).readValue(connectionsString);

        } catch (EncryptionRecoverableException | JsonProcessingException e) {
            throw new RuntimeException(e);
        }
    }
}
