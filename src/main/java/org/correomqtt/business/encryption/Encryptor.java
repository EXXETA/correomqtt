package org.correomqtt.business.encryption;

import org.correomqtt.business.model.PasswordsDTO;
import org.correomqtt.business.provider.EncryptionRecoverableException;

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
