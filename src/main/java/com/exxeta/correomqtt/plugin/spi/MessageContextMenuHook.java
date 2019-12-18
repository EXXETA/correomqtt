package com.exxeta.correomqtt.plugin.spi;

import com.exxeta.correomqtt.plugin.model.MessageExtensionDTO;

public interface MessageContextMenuHook extends BaseExtensionPoint {

    void onCopyMessageToPublishForm(String connectionId, MessageExtensionDTO message);
}
