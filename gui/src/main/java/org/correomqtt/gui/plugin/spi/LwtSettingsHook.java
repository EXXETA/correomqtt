package org.correomqtt.gui.plugin.spi;

import javafx.scene.layout.HBox;
import org.correomqtt.core.plugin.spi.BaseExtensionPoint;
import org.correomqtt.core.plugin.model.LwtConnectionExtensionDTO;

public interface LwtSettingsHook extends BaseExtensionPoint<Object> {

    void onAddItemsToLwtSettingsBox(OnSettingsChangedListener settingsChangeListener, HBox pluginWidgetArea);

    LwtConnectionExtensionDTO onLoadConnection(LwtConnectionExtensionDTO activeConnectionConfigDTO);

    LwtConnectionExtensionDTO onShowConnection(LwtConnectionExtensionDTO activeConnectionConfigDTO);

    LwtConnectionExtensionDTO onSaveConnection(LwtConnectionExtensionDTO activeConnectionConfigDTO);

    LwtConnectionExtensionDTO onUnloadConnection(LwtConnectionExtensionDTO activeConnectionConfigDTO);

    interface OnSettingsChangedListener {
        void setDirty(boolean dirty);
    }
}
