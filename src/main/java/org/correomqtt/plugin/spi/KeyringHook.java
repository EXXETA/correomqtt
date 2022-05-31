package org.correomqtt.plugin.spi;

import org.correomqtt.business.keyring.Keyring;

public interface KeyringHook extends Keyring, BaseExtensionPoint<Object> {
}

