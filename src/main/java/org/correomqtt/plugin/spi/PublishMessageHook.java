package org.correomqtt.plugin.spi;

import com.exxeta.correomqtt.plugin.model.MessageExtensionDTO;

public interface PublishMessageHook extends BaseExtensionPoint {

    MessageExtensionDTO onPublishMessage(String connectionId, MessageExtensionDTO message);

}
