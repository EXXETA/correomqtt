package com.exxeta.correomqtt.plugin.spi;

import com.exxeta.correomqtt.plugin.model.MessageExtensionDTO;

public interface MessageIncomingHook extends BaseExtensionPoint {

    MessageExtensionDTO onMessageIncoming(String connectionId, MessageExtensionDTO messagePropertiesDTO);

}
