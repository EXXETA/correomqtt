package org.correomqtt.plugin.spi;

import org.correomqtt.plugin.model.MessageExtensionDTO;
import javafx.scene.layout.HBox;

public interface MessageListHook extends BaseExtensionPoint<Object> {

    void onCreateEntry(MessageExtensionDTO messageDTO, HBox labelArea);

}
