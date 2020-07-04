package org.correomqtt.business.provider;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.correomqtt.business.dispatcher.ConfigDispatcher;
import org.correomqtt.business.dispatcher.SecretStoreDispatcher;
import org.correomqtt.business.keyring.KeyringException;
import org.correomqtt.business.model.ConnectionConfigDTO;
import org.correomqtt.business.model.ConnectionPasswordType;
import org.correomqtt.business.model.PasswordsDTO;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import javax.crypto.Cipher;
import javax.crypto.SecretKey;
import javax.crypto.SecretKeyFactory;
import javax.crypto.spec.IvParameterSpec;
import javax.crypto.spec.PBEKeySpec;
import javax.crypto.spec.SecretKeySpec;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.AccessDeniedException;
import java.nio.file.DirectoryNotEmptyException;
import java.nio.file.FileAlreadyExistsException;
import java.nio.file.InvalidPathException;
import java.security.AlgorithmParameters;
import java.security.GeneralSecurityException;
import java.security.NoSuchAlgorithmException;
import java.security.spec.InvalidKeySpecException;
import java.util.Base64;
import java.util.HashMap;
import java.util.Map;
import java.util.UUID;


public class SecretStoreProvider extends BaseUserFileProvider {

    private static final Logger LOGGER = LoggerFactory.getLogger(SecretStoreProvider.class);

    private static final String PASSWORD_FILE_NAME = "passwords.json";
    private static final String EX_MSG_PREPARE_CONFIG = "Exception preparing password file.";

    private static final int ITERATION_COUNT = 40000;
    private static final int KEY_LENGTH = 128;

    private PasswordsDTO passwordsDTO;
    private Map<String, String> decryptedPasswords;

    private static SecretStoreProvider instance = null;

    public SecretStoreProvider() {

        try {
            prepareFile(PASSWORD_FILE_NAME);
        } catch (InvalidPathException e) {
            LOGGER.error(EX_MSG_PREPARE_CONFIG, e);
            ConfigDispatcher.getInstance().onInvalidPath();
        } catch (FileAlreadyExistsException e) {
            LOGGER.error(EX_MSG_PREPARE_CONFIG, e);
            ConfigDispatcher.getInstance().onFileAlreadyExists();
        } catch (DirectoryNotEmptyException e) {
            LOGGER.error(EX_MSG_PREPARE_CONFIG, e);
            ConfigDispatcher.getInstance().onConfigDirectoryEmpty();
        } catch (SecurityException | AccessDeniedException e) {
            LOGGER.error(EX_MSG_PREPARE_CONFIG, e);
            ConfigDispatcher.getInstance().onConfigDirectoryNotAccessible();
        } catch (UnsupportedOperationException | IOException e) {
            LOGGER.error(EX_MSG_PREPARE_CONFIG, e);
            ConfigDispatcher.getInstance().onConfigPrepareFailure();
        }

        try {
            passwordsDTO = new ObjectMapper().readValue(this.getFile(), PasswordsDTO.class);
        } catch (IOException e) {
            LOGGER.error("Password file can not be read. ", e);
            SecretStoreDispatcher.getInstance().onPasswordFileUnreadable();
            passwordsDTO = new PasswordsDTO();
        }
        if (passwordsDTO.getSalt() == null) {
            passwordsDTO.setSalt(UUID.randomUUID().toString());
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

    public void setPassword(String masterPassword, ConnectionConfigDTO connection, ConnectionPasswordType type, String password) throws PasswordRecoverableException {
        getDecryptedPasswords(masterPassword).put(getPasswordKey(connection, type), password);
    }

    public String getPassword(String masterPassword, ConnectionConfigDTO connection, ConnectionPasswordType type) throws PasswordRecoverableException {
        return getDecryptedPasswords(masterPassword).get(getPasswordKey(connection, type));
    }

    private String getPasswordKey(ConnectionConfigDTO connection, ConnectionPasswordType type) {
        return connection.getId() + "_" + type.getLabel();
    }

    public void encryptAndSavePasswords(String masterPassword) throws PasswordRecoverableException {

        if (masterPassword == null || masterPassword.isEmpty()) {
            LOGGER.error("Password must not be empty.");
            throw new PasswordRecoverableException();
        }

        Map<String, String> decryptedPasswords = getDecryptedPasswords(masterPassword);

        try {
            String encryptedPasswords = "";
            if (decryptedPasswords.size() != 0) {
                encryptedPasswords = encrypt(new ObjectMapper().writeValueAsString(decryptedPasswords), createSecretKey(masterPassword));
            }
            passwordsDTO.setPasswords(encryptedPasswords);
            new ObjectMapper().writeValue(getFile(), passwordsDTO);
        } catch (GeneralSecurityException e) {
            LOGGER.error("Could not encrypt passwords. ", e);
            throw new PasswordRecoverableException();
        } catch (IOException e) {
            LOGGER.error("Could not save encrypted passwords. ", e);
            throw new PasswordRecoverableException();
        }
    }

    private Map<String, String> getDecryptedPasswords(String masterPassword) throws PasswordRecoverableException {
        if(decryptedPasswords == null) {
            if (passwordsDTO.getPasswords() == null) {
                decryptedPasswords = new HashMap<>();
            } else {
                decryptedPasswords = decryptPasswords(masterPassword);
            }
        }
        return decryptedPasswords;
    }

    private Map<String, String> decryptPasswords(String masterPassword) throws PasswordRecoverableException {
        String encryptedPasswords = passwordsDTO.getPasswords();
        if (masterPassword == null || masterPassword.isEmpty()) {
            LOGGER.error("Password must not be empty.");
            throw new PasswordRecoverableException();
        }

        try {
            if (encryptedPasswords == null || encryptedPasswords.isEmpty()) {
                return new HashMap<>();
            }
            return new ObjectMapper().readValue(decrypt(encryptedPasswords, createSecretKey(masterPassword)), new TypeReference<HashMap<String, String>>() {
            });
        } catch (GeneralSecurityException e) {
            LOGGER.error("Could not decrypt passwords. ", e);
            throw new PasswordRecoverableException();
        } catch (JsonProcessingException e) {
            LOGGER.error("Could not read password file. ", e);
            throw new PasswordRecoverableException();
        }
    }

    public void wipe() {
        decryptedPasswords = null;
        passwordsDTO.setSalt(UUID.randomUUID().toString());
        passwordsDTO.setPasswords("");
        if(getFile().exists() && !getFile().delete()){
            throw new KeyringException("Could not delete passwords.json file.");
        }
    }


    private SecretKeySpec createSecretKey(String masterpassword) throws NoSuchAlgorithmException, InvalidKeySpecException {
        SecretKeyFactory keyFactory = SecretKeyFactory.getInstance("PBKDF2WithHmacSHA512");
        PBEKeySpec keySpec = new PBEKeySpec(masterpassword.toCharArray(), this.passwordsDTO.getSalt().getBytes(StandardCharsets.UTF_8), ITERATION_COUNT, KEY_LENGTH);
        SecretKey keyTmp = keyFactory.generateSecret(keySpec);
        return new SecretKeySpec(keyTmp.getEncoded(), "AES");
    }

    private String encrypt(String passwordToEncrypt, SecretKeySpec keyspec) throws GeneralSecurityException {
        Cipher pbeCipher = Cipher.getInstance("AES/CBC/PKCS5Padding");
        pbeCipher.init(Cipher.ENCRYPT_MODE, keyspec);
        AlgorithmParameters parameters = pbeCipher.getParameters();
        IvParameterSpec ivParameterSpec = parameters.getParameterSpec(IvParameterSpec.class);
        byte[] cryptoText = pbeCipher.doFinal(passwordToEncrypt.getBytes(StandardCharsets.UTF_8));
        byte[] iv = ivParameterSpec.getIV();
        return Base64.getEncoder().encodeToString(iv) + ":" + Base64.getEncoder().encodeToString(cryptoText);
    }


    private String decrypt(String passwordToDecrypt, SecretKeySpec keyspec) throws GeneralSecurityException {
        String iv = passwordToDecrypt.split(":")[0];
        String property = passwordToDecrypt.split(":")[1];
        Cipher pbeCipher = Cipher.getInstance("AES/CBC/PKCS5Padding");
        pbeCipher.init(Cipher.DECRYPT_MODE, keyspec, new IvParameterSpec(Base64.getDecoder().decode(iv)));
        return new String(pbeCipher.doFinal(Base64.getDecoder().decode(property)), StandardCharsets.UTF_8);
    }

    public void ensurePasswordsAreDecrypted(String masterPassword) throws PasswordRecoverableException {
        getDecryptedPasswords(masterPassword);
    }
}
