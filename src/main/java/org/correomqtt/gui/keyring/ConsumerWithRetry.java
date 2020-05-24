package org.correomqtt.gui.keyring;

import org.correomqtt.business.provider.PasswordRecoverableException;

@FunctionalInterface
public interface ConsumerWithRetry {
    void apply(String t) throws PasswordRecoverableException;
}