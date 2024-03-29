package org.correomqtt.core.importexport.connections;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.correomqtt.di.Assisted;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import org.correomqtt.core.concurrent.NoProgressTask;
import org.correomqtt.core.concurrent.TaskException;
import org.correomqtt.core.encryption.EncryptorAesGcm;
import org.correomqtt.di.SoyEvents;
import org.correomqtt.core.fileprovider.EncryptionRecoverableException;
import org.correomqtt.core.model.ConnectionConfigDTO;

import java.util.List;

@DefaultBean
public class ImportDecryptConnectionsTask extends NoProgressTask<List<ConnectionConfigDTO>, ImportDecryptConnectionsTask.Error> {


    public enum Error {
        ENCRYPTION_TYPE_NOT_ALLOWED,
        FILE_CAN_NOT_BE_READ_OR_PARSED
    }

    private final String encryptedData;
    private final String encryptionType;
    private final String password;



    @Inject
    public ImportDecryptConnectionsTask(SoyEvents soyEvents,
                                        @Assisted("encryptedData") String encryptedData,
                                        @Assisted("encryptedType") String encryptionType,
                                        @Assisted("password") String password) {
        super(soyEvents);
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
