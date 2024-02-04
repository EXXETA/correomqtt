package org.correomqtt.gui.plugin.spi;

import org.correomqtt.core.plugin.spi.BaseExtensionPoint;

public interface DetailViewManipulatorHook extends BaseExtensionPoint<Object> {

    byte[] manipulate(byte[] selection);
}
