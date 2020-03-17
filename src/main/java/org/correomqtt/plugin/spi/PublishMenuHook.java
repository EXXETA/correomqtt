package org.correomqtt.plugin.spi;

import javafx.scene.layout.HBox;

public interface PublishMenuHook extends BaseExtensionPoint {

    void onInstantiatePublishMenu(String connectionId, HBox pluginWidgetArea);
}
