package org.correomqtt.gui.transformer;

import javafx.collections.FXCollections;
import org.correomqtt.business.model.MessageDTO;
import org.correomqtt.gui.model.MessagePropertiesDTO;
import org.correomqtt.plugin.model.MessageExtensionDTO;

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
    public static MessagePropertiesDTO mergeProps(MessageExtensionDTO from, MessagePropertiesDTO to) {
        to.getTopicProperty().setValue(from.getTopic());
        to.getPayloadProperty().setValue(from.getPayload());
        to.getIsRetainedProperty().setValue(from.isRetained());
        to.getQosProperty().setValue(from.getQos());
        to.getDateTimeProperty().setValue(from.getDateTime());
        to.getMessageIdProperty().setValue(from.getMessageId());
        to.getMessageTypeProperty().setValue(from.getMessageType());
        to.getPublishStatusProperty().setValue(from.getPublishStatus());
        to.getExtraProperties().setValue(FXCollections.observableMap(from.getCustomFields()));
        return to;
    }
}
