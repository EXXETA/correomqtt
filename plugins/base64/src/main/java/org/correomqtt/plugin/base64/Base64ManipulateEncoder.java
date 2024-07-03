package org.correomqtt.plugin.base64;

import org.correomqtt.core.plugin.spi.ExtensionId;
import org.correomqtt.gui.plugin.spi.DetailViewManipulatorHook;
import org.pf4j.Extension;

@Extension
@ExtensionId("encode")
public class Base64ManipulateEncoder implements DetailViewManipulatorHook {

    @Override
    public byte[] manipulate(byte[] bytes) {
        return Base64Utils.encode(bytes);
    }
}
