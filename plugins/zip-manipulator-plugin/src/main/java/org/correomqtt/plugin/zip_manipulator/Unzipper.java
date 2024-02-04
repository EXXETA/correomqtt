package org.correomqtt.plugin.zip_manipulator;

import org.correomqtt.core.plugin.spi.ExtensionId;
import org.correomqtt.gui.plugin.spi.DetailViewManipulatorHook;
import org.pf4j.Extension;

@Extension
@ExtensionId("unzip")
public class Unzipper implements DetailViewManipulatorHook {

    @Override
    public byte[] manipulate(byte[] bytes) {
        return ZipUtils.unzip(bytes);
    }
}
