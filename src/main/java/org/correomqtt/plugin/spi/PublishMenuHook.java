package org.correomqtt.plugin.spi;

import javafx.scene.layout.HBox;

public interface PublishMenuHook extends BaseExtensionPoint<Object> {

    void onInstantiatePublishMenu(String connectionId, HBox pluginWidgetArea);
}
