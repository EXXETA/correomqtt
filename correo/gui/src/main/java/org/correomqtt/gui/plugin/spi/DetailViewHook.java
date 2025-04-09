package org.correomqtt.gui.plugin.spi;

import javafx.scene.layout.HBox;
import org.correomqtt.core.plugin.spi.BaseExtensionPoint;
import org.correomqtt.core.plugin.model.MessageExtensionDTO;

public interface DetailViewHook extends BaseExtensionPoint<Object> {

    void onOpenDetailView(MessageExtensionDTO messageDTO, HBox labelArea);
}
