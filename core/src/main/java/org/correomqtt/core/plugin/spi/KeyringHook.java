package org.correomqtt.core.plugin.spi;

import org.correomqtt.core.keyring.Keyring;
import org.correomqtt.core.plugin.spi.BaseExtensionPoint;

public interface KeyringHook extends Keyring, BaseExtensionPoint<Object> {
}

