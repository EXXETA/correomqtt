package org.correomqtt.plugin.spi;

import org.correomqtt.plugin.model.MessageExtensionDTO;

public interface OutgoingMessageHook extends BaseExtensionPoint<Object> {

    MessageExtensionDTO onPublishMessage(String connectionId, MessageExtensionDTO message);

}
