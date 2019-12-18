package com.exxeta.correomqtt.plugin.spi;

public interface DetailViewManipulatorHook extends BaseExtensionPoint {

    byte[] manipulate(byte[] selection);
}
