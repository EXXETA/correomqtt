package org.correomqtt.business.services;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.correomqtt.business.dispatcher.ImportDecryptConnectionsDispatcher;
import org.correomqtt.business.encryption.EncryptorAesGcm;
import org.correomqtt.business.exception.CorreoMqttExecutionException;
import org.correomqtt.business.model.ConnectionConfigDTO;
import org.correomqtt.business.provider.EncryptionRecoverableException;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.List;

public class ImportDecryptConnectionsService implements BusinessService {


    private static final Logger LOGGER = LoggerFactory.getLogger(ImportDecryptConnectionsService.class);
    private final String encryptedData;
    private final String password;
    private final String encryptionType;
    private List<ConnectionConfigDTO> decryptedConnectionList;

    public ImportDecryptConnectionsService(String encryptedData, String encryptionType, String password) {
        this.encryptedData = encryptedData;
        this.encryptionType = encryptionType;
        this.password = password;
    }

    public void decrypt() {

        EncryptorAesGcm encryptor = new EncryptorAesGcm(password);
        if (!encryptor.getEncryptionTranslation().equals(encryptionType)) {
            throw new CorreoMqttExecutionException(new RuntimeException("Encryption Type is not allowed"));
        }

        try {
            String connectionsString = encryptor.decrypt(this.encryptedData);
            decryptedConnectionList = new ObjectMapper().readerFor(new TypeReference<List<ConnectionConfigDTO>>() {
            }).readValue(connectionsString);

        } catch (EncryptionRecoverableException | JsonProcessingException e) {
            throw new CorreoMqttExecutionException(e);
        }
    }


    @Override
    public void onSucceeded() {
        LOGGER.info("Decrypting imported connections succeeded.");
        ImportDecryptConnectionsDispatcher.getInstance().onDecryptSucceeded(decryptedConnectionList);
    }

    @Override
    public void onCancelled() {
        LOGGER.info("Decrypting imported connections cancelled.");
        ImportDecryptConnectionsDispatcher.getInstance().onDecryptCancelled();
    }

    @Override
    public void onFailed(Throwable exception) {
        LOGGER.info("Decrypting imported connections failed.", exception);
        ImportDecryptConnectionsDispatcher.getInstance().onDecryptFailed(exception);

    }

    @Override
    public void onRunning() {
        LOGGER.info("Decrypting imported connections running.");
        ImportDecryptConnectionsDispatcher.getInstance().onDecryptRunning();

    }

    @Override
    public void onScheduled() {
        LOGGER.info("Decrypting imported connections scheduled.");
        ImportDecryptConnectionsDispatcher.getInstance().onDecryptScheduled();

    }
}
