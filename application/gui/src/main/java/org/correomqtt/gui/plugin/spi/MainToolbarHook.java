package org.correomqtt.gui.plugin.spi;

import javafx.scene.layout.HBox;
import org.correomqtt.core.plugin.spi.BaseExtensionPoint;

public interface MainToolbarHook extends BaseExtensionPoint<Object> {

    void onInstantiateMainToolbar(String connectionId, HBox controllViewButtonHBox, int indexToInsert);

}
