package org.correomqtt.plugin.spi;

import org.correomqtt.plugin.model.MessageExtensionDTO;

public interface IncomingMessageHook extends BaseExtensionPoint {

    MessageExtensionDTO onMessageIncoming(String connectionId, MessageExtensionDTO messagePropertiesDTO);

}
