package org.correomqtt.plugin.spi;

import org.correomqtt.plugin.model.MessageExtensionDTO;

public interface OutgoingMessageHook<T extends OutgoingMessageHookDTO> extends BaseExtensionPoint<T> {

    MessageExtensionDTO onPublishMessage(String connectionId, MessageExtensionDTO message);

    T getConfig();

    void onConfigReceived(T config);
}
