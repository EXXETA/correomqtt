package org.correomqtt.core.plugin.spi;

import org.correomqtt.core.keyring.Keyring;

public interface KeyringHook extends Keyring, BaseExtensionPoint<Object> {
}

