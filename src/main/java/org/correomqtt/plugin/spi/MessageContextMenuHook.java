package org.correomqtt.plugin.spi;

import org.correomqtt.plugin.model.MessageExtensionDTO;

public interface MessageContextMenuHook extends BaseExtensionPoint<Object> {

    void onCopyMessageToPublishForm(String connectionId, MessageExtensionDTO message);
}
