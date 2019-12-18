package com.exxeta.correomqtt.plugin.spi;

import com.exxeta.correomqtt.plugin.model.MessageExtensionDTO;

public interface PublishMessageHook extends BaseExtensionPoint {

    MessageExtensionDTO onPublishMessage(String connectionId, MessageExtensionDTO message);

}
