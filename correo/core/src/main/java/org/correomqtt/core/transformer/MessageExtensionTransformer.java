package org.correomqtt.core.transformer;

import org.correomqtt.core.model.MessageDTO;
import org.correomqtt.core.plugin.model.MessageExtensionDTO;

public class MessageExtensionTransformer {

    private MessageExtensionTransformer() {
        //private constructor
    }

    public static MessageDTO mergeDTO(MessageExtensionDTO from, MessageDTO to) {
        to.setTopic(from.getTopic());
        to.setPayload(from.getPayload());
        to.setRetained(from.isRetained());
        to.setQos(from.getQos());
        to.setDateTime(from.getDateTime());
        to.setMessageId(from.getMessageId());
        to.setMessageType(from.getMessageType());
        to.setPublishStatus(from.getPublishStatus());
        return to;
    }
}
