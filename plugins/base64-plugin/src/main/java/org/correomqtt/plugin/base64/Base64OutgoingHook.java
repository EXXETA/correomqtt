package org.correomqtt.plugin.base64;

import org.correomqtt.core.plugin.model.MessageExtensionDTO;
import org.correomqtt.core.plugin.spi.ExtensionId;
import org.correomqtt.core.plugin.spi.OutgoingMessageHook;
import org.pf4j.Extension;

@Extension
@ExtensionId("base64.outgoing")
public class Base64OutgoingHook implements OutgoingMessageHook<Base64OutgoingHookDTO>  {

    private Base64OutgoingHookDTO config;

    @Override
    public void onConfigReceived(Base64OutgoingHookDTO config) {
        this.config = config;
    }

    @Override
    public MessageExtensionDTO onPublishMessage(String connectionId, MessageExtensionDTO extensionMessageDTO) {
        extensionMessageDTO.setPayload(new String(Base64Utils.encode(extensionMessageDTO.getPayload().getBytes())));
        return extensionMessageDTO;
    }

    @Override
    public Base64OutgoingHookDTO getConfig() {
        return config;
    }

    @Override
    public Class<Base64OutgoingHookDTO> getConfigClass(){
        return Base64OutgoingHookDTO.class;
    }
}
