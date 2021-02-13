package org.correomqtt.plugin.spi;

import javafx.scene.layout.HBox;

public interface MainToolbarHook extends BaseExtensionPoint {

    void onInstantiateMainToolbar(String connectionId, HBox pluginWidgetArea);



}
