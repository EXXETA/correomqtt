package org.correomqtt.plugin.spi;

import org.correomqtt.plugin.model.MessageExtensionDTO;

public interface MessageIncomingHook extends BaseExtensionPoint {

    MessageExtensionDTO onMessageIncoming(String connectionId, MessageExtensionDTO messagePropertiesDTO);

}
