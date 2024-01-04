package org.correomqtt.gui.keyring;

import org.correomqtt.business.fileprovider.EncryptionRecoverableException;

@FunctionalInterface
public interface ConsumerWithRetry {
    void apply(String t) throws EncryptionRecoverableException;
}