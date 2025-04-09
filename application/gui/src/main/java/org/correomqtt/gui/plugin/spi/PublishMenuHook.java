package org.correomqtt.gui.plugin.spi;

import javafx.scene.layout.HBox;
import org.correomqtt.core.plugin.spi.BaseExtensionPoint;

public interface PublishMenuHook extends BaseExtensionPoint<Object> {

    void onInstantiatePublishMenu(String connectionId, HBox pluginWidgetArea);
}
