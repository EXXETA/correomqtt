package org.correomqtt.gui.keyring;

import org.correomqtt.business.provider.EncryptionRecoverableException;

@FunctionalInterface
public interface ConsumerWithRetry {
    void apply(String t) throws EncryptionRecoverableException;
}