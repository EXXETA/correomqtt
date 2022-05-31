package org.correomqtt.plugin.spi;

public interface DetailViewManipulatorHook extends BaseExtensionPoint<Object> {

    byte[] manipulate(byte[] selection);
}
