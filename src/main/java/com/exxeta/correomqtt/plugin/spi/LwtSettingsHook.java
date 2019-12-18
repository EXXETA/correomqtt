package com.exxeta.correomqtt.plugin.spi;

import com.exxeta.correomqtt.plugin.model.LwtConnectionExtensionDTO;
import javafx.scene.layout.HBox;

public interface LwtSettingsHook extends BaseExtensionPoint {

    void onAddItemsToLwtSettingsBox(OnSettingsChangedListener settingsChangeListener, HBox pluginWidgetArea);

    LwtConnectionExtensionDTO onLoadConnection(LwtConnectionExtensionDTO activeConnectionConfigDTO);

    LwtConnectionExtensionDTO onShowConnection(LwtConnectionExtensionDTO activeConnectionConfigDTO);

    LwtConnectionExtensionDTO onSaveConnection(LwtConnectionExtensionDTO activeConnectionConfigDTO);

    LwtConnectionExtensionDTO onUnloadConnection(LwtConnectionExtensionDTO activeConnectionConfigDTO);

    interface OnSettingsChangedListener {
        void setDirty(boolean dirty);
    }
}
