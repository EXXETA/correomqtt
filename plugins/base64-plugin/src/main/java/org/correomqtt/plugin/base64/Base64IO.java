package org.correomqtt.plugin.base64;

import org.correomqtt.core.plugin.model.MessageExtensionDTO;
import org.correomqtt.core.plugin.spi.ExtensionId;
import org.correomqtt.core.plugin.spi.IncomingMessageHookDTO;
import org.correomqtt.core.plugin.spi.OutgoingMessageHook;
import org.correomqtt.core.plugin.spi.OutgoingMessageHookDTO;
import org.pf4j.Extension;

@Extension
@ExtensionId("base64.io")
public class Base64IO implements OutgoingMessageHook/*, IncomingMessageHook*/ {

    private Base64IOConfigDTO config;

  //  @Override
    public MessageExtensionDTO onMessageIncoming(String connectionId, MessageExtensionDTO extensionMessageDTO) {
        extensionMessageDTO.setPayload(new String(Base64Utils.decode(extensionMessageDTO.getPayload().getBytes())));
        return extensionMessageDTO;
    }
//TODO
  //  @Override
    public void onConfigReceived(IncomingMessageHookDTO config) {

    }

    @Override
    public MessageExtensionDTO onPublishMessage(String connectionId, MessageExtensionDTO extensionMessageDTO) {
        extensionMessageDTO.setPayload(new String(Base64Utils.encode(extensionMessageDTO.getPayload().getBytes())));
        return extensionMessageDTO;
    }

    @Override
    public OutgoingMessageHookDTO getConfig() {
        return null;
    }


    @Override
    public void onConfigReceived(OutgoingMessageHookDTO config) {

    }
}
