package org.correomqtt.plugin.spi;

import org.correomqtt.plugin.model.MessageExtensionDTO;
import javafx.scene.layout.HBox;

public interface DetailViewHook extends BaseExtensionPoint {

    void onOpenDetailView(MessageExtensionDTO messageDTO, HBox labelArea);
}
