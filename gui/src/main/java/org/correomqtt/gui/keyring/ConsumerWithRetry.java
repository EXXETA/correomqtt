package org.correomqtt.gui.keyring;

import org.correomqtt.core.fileprovider.EncryptionRecoverableException;

@FunctionalInterface
public interface ConsumerWithRetry {
    void apply(String t) throws EncryptionRecoverableException;
}