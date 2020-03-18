package org.correomqtt.gui.transformer;

import org.correomqtt.business.model.MessageDTO;
import org.correomqtt.gui.model.MessagePropertiesDTO;

public class MessageTransformer {

    private MessageTransformer() {
        //private constructor
    }

    public static MessagePropertiesDTO dtoToProps(MessageDTO messageDTO) {
        return MessagePropertiesDTO.builder()
                .topic(messageDTO.getTopic())
                .payload(messageDTO.getPayload())
                .isRetained(messageDTO.isRetained())
                .qos(messageDTO.getQos())
                .dateTime(messageDTO.getDateTime())
                .messageId(messageDTO.getMessageId())
                .messageType(messageDTO.getMessageType())
                .publishStatus(messageDTO.getPublishStatus())
                .build();
    }

    public static MessageDTO propsToDTO(MessagePropertiesDTO messagePropertiesDTO) {
        return MessageDTO.builder()
                .topic(messagePropertiesDTO.getTopic())
                .payload(messagePropertiesDTO.getPayload())
                .isRetained(messagePropertiesDTO.isRetained())
                .qos(messagePropertiesDTO.getQos())
                .dateTime(messagePropertiesDTO.getDateTime())
                .messageId(messagePropertiesDTO.getMessageId())
                .messageType(messagePropertiesDTO.getMessageType())
                .publishStatus(messagePropertiesDTO.getPublishStatus())
                .build();
    }
}
