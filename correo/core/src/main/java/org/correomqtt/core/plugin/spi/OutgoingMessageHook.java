package org.correomqtt.core.plugin.spi;

import org.correomqtt.core.plugin.model.MessageExtensionDTO;

public interface OutgoingMessageHook<T extends OutgoingMessageHookDTO> extends BaseExtensionPoint<T> {

    MessageExtensionDTO onPublishMessage(String connectionId, MessageExtensionDTO message);

    @Override
    T getConfig();

    @Override
    void onConfigReceived(T config);

}
