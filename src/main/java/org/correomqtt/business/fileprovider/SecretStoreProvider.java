package org.correomqtt.business.fileprovider;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.correomqtt.business.encryption.Encryptor;
import org.correomqtt.business.encryption.EncryptorAesCbc;
import org.correomqtt.business.encryption.EncryptorAesGcm;
import org.correomqtt.business.eventbus.EventBus;
import org.correomqtt.business.keyring.KeyringException;
import org.correomqtt.business.model.ConnectionConfigDTO;
import org.correomqtt.business.model.ConnectionPasswordType;
import org.correomqtt.business.model.PasswordsDTO;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.InvalidPathException;
import java.nio.file.Path;
import java.util.HashMap;
import java.util.Map;


public class SecretStoreProvider extends BaseUserFileProvider {

    private static final Logger LOGGER = LoggerFactory.getLogger(SecretStoreProvider.class);

    private static final String PASSWORD_FILE_NAME = "passwords.json";

    private PasswordsDTO passwordsDTO;
    private Map<String, String> decryptedPasswords;

    private static SecretStoreProvider instance = null;

    public SecretStoreProvider() {

        try {
            prepareFile(PASSWORD_FILE_NAME);
        } catch (InvalidPathException | SecurityException | UnsupportedOperationException | IOException e) {
            LOGGER.error("Error writing passwords file {}. ", PASSWORD_FILE_NAME, e);
            EventBus.fire(new UnaccessiblePasswordFileEvent(e));
        }

        try {
            passwordsDTO = new ObjectMapper().readValue(this.getFile(), PasswordsDTO.class);
        } catch (IOException e) {
            LOGGER.error("Password file can not be read {}.", PASSWORD_FILE_NAME, e);
            EventBus.fire(new InvalidPasswordFileEvent());
            passwordsDTO = new PasswordsDTO();
        }

    }

    public static synchronized SecretStoreProvider getInstance() {
        if (instance == null) {
            instance = new SecretStoreProvider();
            return instance;
        } else {
            return instance;
        }
    }

    public void setPassword(String masterPassword, ConnectionConfigDTO connection, ConnectionPasswordType type, String password) throws EncryptionRecoverableException {
        readDecryptedPasswords(getEncryptor(masterPassword)).put(getPasswordKey(connection, type), password);
    }

    public String getPassword(String masterPassword, ConnectionConfigDTO connection, ConnectionPasswordType type) throws EncryptionRecoverableException {
        return readDecryptedPasswords(getEncryptor(masterPassword)).get(getPasswordKey(connection, type));
    }

    private String getPasswordKey(ConnectionConfigDTO connection, ConnectionPasswordType type) {
        return connection.getId() + "_" + type.getLabel();
    }

    public void encryptAndSavePasswords(String masterPassword) throws EncryptionRecoverableException {

        Encryptor encryptor = getEncryptor(masterPassword);

        Map<String, String> localDecryptedPasswords = readDecryptedPasswords(encryptor);

        try {
            String encryptedPasswords = "";
            if (!localDecryptedPasswords.isEmpty()) {
                encryptedPasswords = encryptor.encrypt(new ObjectMapper().writerWithDefaultPrettyPrinter().writeValueAsString(localDecryptedPasswords));
            }
            passwordsDTO.setSalt(null);
            passwordsDTO.setPasswords(encryptedPasswords);
            passwordsDTO.setEncryptionType(encryptor.getEncryptionTranslation());
            new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(getFile(), passwordsDTO);
        } catch (IOException e) {
            LOGGER.error("Could not save encrypted passwords. ", e);
            throw new EncryptionRecoverableException();
        }
    }

    private Encryptor getEncryptor(String masterPassword) throws EncryptionRecoverableException {

        if (masterPassword == null || masterPassword.isEmpty()) {
            LOGGER.error("Password must not be empty.");
            throw new EncryptionRecoverableException();
        }

        return new EncryptorAesGcm(masterPassword);
    }

    private Map<String, String> readDecryptedPasswords(Encryptor encryptor) throws EncryptionRecoverableException {
        if (decryptedPasswords == null) {
            if (passwordsDTO.getPasswords() == null) {
                decryptedPasswords = new HashMap<>();
            } else {
                decryptedPasswords = decryptPasswords(encryptor);
            }
        }
        return decryptedPasswords;
    }

    @SuppressWarnings("removal")
    private Map<String, String> decryptPasswords(Encryptor encryptor) throws EncryptionRecoverableException {
        String encryptedPasswords = encryptor.passwordsDTOtoString(passwordsDTO);

        try {
            if (encryptedPasswords == null || encryptedPasswords.isEmpty()) {
                return new HashMap<>();
            }
            return new ObjectMapper().readValue(encryptor.decrypt(encryptedPasswords), new TypeReference<HashMap<String, String>>() {
            });
        } catch (JsonProcessingException e) {
            LOGGER.error("Could not read password file. ", e);
            throw new EncryptionRecoverableException();
        }
    }

    public void wipe() {
        decryptedPasswords = null;
        passwordsDTO.setSalt(null);
        passwordsDTO.setPasswords(null);
        passwordsDTO.setEncryptionType(null);
        Path path = getFile().toPath();
        if (Files.exists(path)) {
            try {
                Files.delete(getFile().toPath());
            } catch (IOException e) {
                throw new KeyringException("Could not delete passwords.json file.", e);
            }
        }
    }

    public void ensurePasswordsAreDecrypted(String masterPassword) throws EncryptionRecoverableException {
        readDecryptedPasswords(getEncryptor(masterPassword));
    }

    @SuppressWarnings("removal")
    public void migratePasswordEncryption(String masterPassword) throws EncryptionRecoverableException {
        /* Explanation why removal warning is suppressed.
         * While EncryptorAesCbc is deprecated caused by security we still need the implementation to decrypt and
         * migrate existing passwords.
         */
        if (passwordsDTO != null && passwordsDTO.getPasswords() != null && (passwordsDTO.getEncryptionType() == null || EncryptorAesCbc.ENCRYPTION_TRANSFORMATION.equals(passwordsDTO.getEncryptionType()))) {
            LOGGER.info("Migrating password encryption from {} to {}", EncryptorAesCbc.ENCRYPTION_TRANSFORMATION, EncryptorAesGcm.ENCRYPTION_TRANSFORMATION);
            readDecryptedPasswords(new EncryptorAesCbc(masterPassword));
            encryptAndSavePasswords(masterPassword);
        } else if (passwordsDTO != null && passwordsDTO.getPasswords() != null && EncryptorAesGcm.ENCRYPTION_TRANSFORMATION.equals(passwordsDTO.getEncryptionType())) {
            LOGGER.info("Current password encryption is {}.", EncryptorAesGcm.ENCRYPTION_TRANSFORMATION);
        } else if (passwordsDTO == null || passwordsDTO.getPasswords() == null) {
            LOGGER.info("No passwords are stored currently.");
        } else {
            LOGGER.warn("Unknown password encryption: {}", passwordsDTO.getEncryptionType());
            throw new EncryptionRecoverableException("Unknown password encryption.");
        }
    }
}
