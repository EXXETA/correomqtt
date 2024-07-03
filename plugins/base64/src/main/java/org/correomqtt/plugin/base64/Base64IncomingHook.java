package org.correomqtt.plugin.base64;

import org.correomqtt.core.plugin.model.MessageExtensionDTO;
import org.correomqtt.core.plugin.spi.ExtensionId;
import org.correomqtt.core.plugin.spi.IncomingMessageHook;
import org.pf4j.Extension;

@Extension
@ExtensionId("base64.incoming")
public class Base64IncomingHook implements IncomingMessageHook<Base64IncomingHookDTO> {

    private Base64IncomingHookDTO config;

    @Override
    public MessageExtensionDTO onMessageIncoming(String connectionId, MessageExtensionDTO extensionMessageDTO) {
        extensionMessageDTO.setPayload(new String(Base64Utils.decode(extensionMessageDTO.getPayload().getBytes())));
        return extensionMessageDTO;
    }

    @Override
    public Base64IncomingHookDTO getConfig() {
        return config;
    }

    @Override
    public void onConfigReceived(Base64IncomingHookDTO config) {
        this.config = config;
    }

    @Override
    public Class<Base64IncomingHookDTO> getConfigClass() {
        return Base64IncomingHookDTO.class;
    }
}
