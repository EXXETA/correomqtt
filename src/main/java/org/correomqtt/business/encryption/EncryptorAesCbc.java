package org.correomqtt.business.encryption;

import org.apache.commons.lang3.NotImplementedException;
import org.correomqtt.business.model.PasswordsDTO;
import org.correomqtt.business.provider.EncryptionRecoverableException;

import javax.crypto.BadPaddingException;
import javax.crypto.Cipher;
import javax.crypto.IllegalBlockSizeException;
import javax.crypto.NoSuchPaddingException;
import javax.crypto.SecretKey;
import javax.crypto.SecretKeyFactory;
import javax.crypto.spec.IvParameterSpec;
import javax.crypto.spec.PBEKeySpec;
import javax.crypto.spec.SecretKeySpec;
import java.nio.charset.StandardCharsets;
import java.security.InvalidAlgorithmParameterException;
import java.security.InvalidKeyException;
import java.security.NoSuchAlgorithmException;
import java.security.spec.InvalidKeySpecException;
import java.util.Base64;

/**
 * @deprecated Will be removed due to security issues. Current implementation is used for migration only.
 */
@Deprecated(since = "0.15.0", forRemoval = true)
public class EncryptorAesCbc implements Encryptor {

    public static final String ENCRYPTION_TRANSFORMATION = "AES/CBC/PKCS5Padding";

    private static final int ITERATION_COUNT = 40000;
    private static final int KEY_LENGTH = 128;

    private final String password;

    public EncryptorAesCbc(String password){
        this.password = password;
    }

    @Override
    public String decrypt(String encryptedData) throws EncryptionRecoverableException {

        try {
            String[] splittedData = encryptedData.split(":");
            String salt = splittedData[0];
            String iv = splittedData[1];
            SecretKeySpec keyspec = createSecretKey(password, salt);
            String property = splittedData[2];
            Cipher pbeCipher = Cipher.getInstance(ENCRYPTION_TRANSFORMATION);
            pbeCipher.init(Cipher.DECRYPT_MODE, keyspec, new IvParameterSpec(Base64.getDecoder().decode(iv)));
            return new String(pbeCipher.doFinal(Base64.getDecoder().decode(property)), StandardCharsets.UTF_8);
        } catch (NoSuchAlgorithmException | InvalidKeySpecException | InvalidKeyException | InvalidAlgorithmParameterException |
                IllegalBlockSizeException | BadPaddingException | NoSuchPaddingException e) {
            throw new EncryptionRecoverableException(e);
        }
    }

    @Override
    public String encrypt(String dataToEncrypt) {
        throw new NotImplementedException("This method must not be used.");
    }

    @Override
    public String passwordsDTOtoString(PasswordsDTO passwordsDTO){
        return passwordsDTO.getSalt() + ":" + passwordsDTO.getPasswords();
    }

    @Override
    public String getEncryptionTranslation() {
        return ENCRYPTION_TRANSFORMATION;
    }

    private SecretKeySpec createSecretKey(String password, String salt) throws NoSuchAlgorithmException, InvalidKeySpecException {
        SecretKeyFactory keyFactory = SecretKeyFactory.getInstance("PBKDF2WithHmacSHA512");
        PBEKeySpec keySpec = new PBEKeySpec(password.toCharArray(), salt.getBytes(StandardCharsets.UTF_8), ITERATION_COUNT, KEY_LENGTH);
        SecretKey keyTmp = keyFactory.generateSecret(keySpec);
        return new SecretKeySpec(keyTmp.getEncoded(), "AES");
    }
}
