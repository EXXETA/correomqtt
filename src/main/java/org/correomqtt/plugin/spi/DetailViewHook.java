package org.correomqtt.plugin.spi;

import javafx.scene.layout.HBox;
import org.correomqtt.plugin.model.MessageExtensionDTO;

public interface DetailViewHook extends BaseExtensionPoint<Object> {

    void onOpenDetailView(MessageExtensionDTO messageDTO, HBox labelArea);
}
