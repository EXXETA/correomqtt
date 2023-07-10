package org.correomqtt.plugin.model;

import org.correomqtt.business.model.MessageDTO;
import org.correomqtt.business.model.MessageType;
import org.correomqtt.business.model.PublishStatus;
import org.correomqtt.business.model.Qos;
import org.correomqtt.gui.model.MessagePropertiesDTO;
import lombok.Getter;
import lombok.Setter;

import java.time.LocalDateTime;
import java.util.HashMap;

@Setter
@Getter
public class MessageExtensionDTO  {

    private String topic;
    private String payload;
    private boolean isRetained;
    private Qos qos;
    private LocalDateTime dateTime;
    private String messageId;
    private MessageType messageType;
    private PublishStatus publishStatus;
    private HashMap<String, Object> customFields;

    public MessageExtensionDTO(MessageDTO messageDTO) {
        this.topic = messageDTO.getTopic();
        this.payload = messageDTO.getPayload();
        this.isRetained = messageDTO.isRetained();
        this.qos = messageDTO.getQos();
        this.dateTime = messageDTO.getDateTime();
        this.messageId = messageDTO.getMessageId();
        this.messageType = messageDTO.getMessageType();
        this.publishStatus = messageDTO.getPublishStatus();
    }

    public MessageExtensionDTO(MessagePropertiesDTO messagePropertiesDTO) {
        this.topic = messagePropertiesDTO.getTopic();
        this.payload = messagePropertiesDTO.getPayload();
        this.isRetained = messagePropertiesDTO.isRetained();
        this.qos = messagePropertiesDTO.getQos();
        this.dateTime = messagePropertiesDTO.getDateTime();
        this.messageId = messagePropertiesDTO.getMessageId();
        this.messageType = messagePropertiesDTO.getMessageType();
        this.publishStatus = messagePropertiesDTO.getPublishStatus();
        this.customFields = new HashMap<>(messagePropertiesDTO.getExtraProperties());
    }
}
