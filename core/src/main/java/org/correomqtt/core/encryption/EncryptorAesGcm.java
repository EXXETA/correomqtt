package org.correomqtt.core.encryption;

import org.correomqtt.core.fileprovider.EncryptionRecoverableException;
import org.correomqtt.core.model.PasswordsDTO;

import javax.crypto.BadPaddingException;
import javax.crypto.Cipher;
import javax.crypto.IllegalBlockSizeException;
import javax.crypto.NoSuchPaddingException;
import javax.crypto.SecretKey;
import javax.crypto.SecretKeyFactory;
import javax.crypto.spec.GCMParameterSpec;
import javax.crypto.spec.PBEKeySpec;
import javax.crypto.spec.SecretKeySpec;
import java.nio.ByteBuffer;
import java.security.InvalidAlgorithmParameterException;
import java.security.InvalidKeyException;
import java.security.NoSuchAlgorithmException;
import java.security.SecureRandom;
import java.security.spec.InvalidKeySpecException;
import java.security.spec.KeySpec;
import java.util.Base64;

import static java.nio.charset.StandardCharsets.UTF_8;


public class EncryptorAesGcm implements Encryptor {

    public static final String ENCRYPTION_TRANSFORMATION = "AES/GCM/NoPadding";

    private static final int TAG_LENGTH_BIT = 128;
    private static final int IV_LENGTH_BYTE = 12;
    private static final int SALT_LENGTH_BYTE = 16;

    private final String password;

    public EncryptorAesGcm(String password) {
        this.password = password;
    }

    private static byte[] getRandomNonce(int length) {
        byte[] nonce = new byte[length];
        new SecureRandom().nextBytes(nonce);
        return nonce;
    }

    @Override
    public String decrypt(String encryptedData) throws EncryptionRecoverableException {

        try {

            byte[] decode = Base64.getDecoder().decode(encryptedData.getBytes(UTF_8));
            ByteBuffer bb = ByteBuffer.wrap(decode);

            byte[] iv = new byte[IV_LENGTH_BYTE];
            bb.get(iv);

            byte[] salt = new byte[SALT_LENGTH_BYTE];
            bb.get(salt);

            byte[] cipherText = new byte[bb.remaining()];
            bb.get(cipherText);

            SecretKey aesKeyFromPassword = getAESKeyFromPassword(salt);
            Cipher cipher = Cipher.getInstance(ENCRYPTION_TRANSFORMATION);
            cipher.init(Cipher.DECRYPT_MODE, aesKeyFromPassword, new GCMParameterSpec(TAG_LENGTH_BIT, iv));
            byte[] plainText = cipher.doFinal(cipherText);
            return new String(plainText, UTF_8);
        } catch (NoSuchAlgorithmException | InvalidKeySpecException | InvalidKeyException | InvalidAlgorithmParameterException |
                IllegalBlockSizeException | BadPaddingException | NoSuchPaddingException e) {
            throw new EncryptionRecoverableException(e);
        }
    }

    @Override
    public String encrypt(String dataToEncrypt) throws EncryptionRecoverableException {
        try {
            byte[] salt = getRandomNonce(SALT_LENGTH_BYTE);
            byte[] iv = getRandomNonce(IV_LENGTH_BYTE);
            SecretKey aesKeyFromPassword = getAESKeyFromPassword(salt);
            Cipher cipher = Cipher.getInstance(ENCRYPTION_TRANSFORMATION);
            cipher.init(Cipher.ENCRYPT_MODE, aesKeyFromPassword, new GCMParameterSpec(TAG_LENGTH_BIT, iv));

            byte[] cipherText = cipher.doFinal(dataToEncrypt.getBytes(UTF_8));

            byte[] cipherTextWithIvSalt = ByteBuffer.allocate(iv.length + salt.length + cipherText.length)
                    .put(iv)
                    .put(salt)
                    .put(cipherText)
                    .array();

            return Base64.getEncoder().encodeToString(cipherTextWithIvSalt);

        } catch (NoSuchAlgorithmException | InvalidKeySpecException | NoSuchPaddingException | InvalidKeyException |
                InvalidAlgorithmParameterException | IllegalBlockSizeException | BadPaddingException e) {
            throw new EncryptionRecoverableException(e);
        }
    }

    @Override
    public String passwordsDTOtoString(PasswordsDTO passwordsDTO) {
        return passwordsDTO.getPasswords();
    }

    private SecretKey getAESKeyFromPassword(byte[] salt)
            throws NoSuchAlgorithmException, InvalidKeySpecException {

        SecretKeyFactory factory = SecretKeyFactory.getInstance("PBKDF2WithHmacSHA256");
        KeySpec spec = new PBEKeySpec(password.toCharArray(), salt, 65536, 256);
        return new SecretKeySpec(factory.generateSecret(spec).getEncoded(), "AES");
    }

    @Override
    public String getEncryptionTranslation() {
        return ENCRYPTION_TRANSFORMATION;
    }
}
