package org.correomqtt.gui.plugin.spi;

import javafx.scene.layout.HBox;
import org.correomqtt.core.plugin.spi.BaseExtensionPoint;
import org.correomqtt.core.plugin.model.MessageExtensionDTO;

public interface MessageListHook extends BaseExtensionPoint<Object> {

    void onCreateEntry(MessageExtensionDTO messageDTO, HBox labelArea);

}
