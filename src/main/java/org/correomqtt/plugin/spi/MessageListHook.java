package org.correomqtt.plugin.spi;

import javafx.scene.layout.HBox;
import org.correomqtt.plugin.model.MessageExtensionDTO;

public interface MessageListHook extends BaseExtensionPoint<Object> {

    void onCreateEntry(MessageExtensionDTO messageDTO, HBox labelArea);

}
