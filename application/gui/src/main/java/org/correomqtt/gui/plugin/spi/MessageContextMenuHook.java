package org.correomqtt.gui.plugin.spi;

import org.correomqtt.core.plugin.spi.BaseExtensionPoint;
import org.correomqtt.core.plugin.model.MessageExtensionDTO;

public interface MessageContextMenuHook extends BaseExtensionPoint<Object> {

    void onCopyMessageToPublishForm(String connectionId, MessageExtensionDTO message);
}
