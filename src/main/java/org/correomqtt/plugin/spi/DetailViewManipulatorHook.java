package org.correomqtt.plugin.spi;

public interface DetailViewManipulatorHook extends BaseExtensionPoint {

    byte[] manipulate(byte[] selection);
}
