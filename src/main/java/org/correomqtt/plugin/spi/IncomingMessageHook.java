package org.correomqtt.plugin.spi;

import org.correomqtt.plugin.model.MessageExtensionDTO;

public interface IncomingMessageHook<T extends IncomingMessageHookDTO> extends BaseExtensionPoint<T> {

    MessageExtensionDTO onMessageIncoming(String connectionId, MessageExtensionDTO messagePropertiesDTO);

    @Override
    T getConfig();

    @Override
    void onConfigReceived(T config);

}
