package org.correomqtt.business.fileprovider;

public class EncryptionRecoverableException extends Exception {

    public EncryptionRecoverableException() {
        super();
    }

    public EncryptionRecoverableException(String message) {
        super(message);
    }

    public EncryptionRecoverableException(Throwable cause) {
        super(cause);
    }

    public EncryptionRecoverableException(String message, Throwable cause) {
        super(message, cause);
    }
}
