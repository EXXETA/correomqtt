package org.correomqtt.core.encryption;

import org.correomqtt.core.fileprovider.EncryptionRecoverableException;
import org.correomqtt.core.model.PasswordsDTO;

public interface Encryptor {

    String decrypt(String encryptedData) throws EncryptionRecoverableException;

    String encrypt(String dataToEncrypt) throws EncryptionRecoverableException;

    /**
     * @deprecated Only used for EncryptorAesCBC
     */
    @Deprecated(since="0.15.0", forRemoval = true)
    default String passwordsDTOtoString(PasswordsDTO passwordsDTO){
        return passwordsDTO.getPasswords();
    }

    String getEncryptionTranslation();
}
