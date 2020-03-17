package org.correomqtt.plugin.spi;

import com.exxeta.correomqtt.plugin.model.MessageExtensionDTO;
import javafx.scene.layout.HBox;

public interface MessageListHook extends BaseExtensionPoint {

    void onCreateEntry(MessageExtensionDTO messageDTO, HBox labelArea);

}
