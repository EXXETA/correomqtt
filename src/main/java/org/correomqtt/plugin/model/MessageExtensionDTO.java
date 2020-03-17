package org.correomqtt.plugin.model;

import org.correomqtt.business.model.MessageType;
import org.correomqtt.business.model.PublishStatus;
import org.correomqtt.business.model.Qos;
import org.correomqtt.gui.model.MessagePropertiesDTO;
import com.sun.javafx.collections.ObservableMapWrapper;
import lombok.Getter;
import lombok.Setter;

import java.time.LocalDateTime;
import java.util.HashMap;

@Setter
@Getter
public class MessageExtensionDTO {

    private String topic;
    private String payload;
    private boolean isRetained;
    private Qos qos;
    private LocalDateTime dateTime;
    private String messageId;
    private MessageType messageType;
    private PublishStatus publishStatus;
    private HashMap<String, Object> customFields;

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

    public MessagePropertiesDTO merge(MessagePropertiesDTO messageDTO) {
        messageDTO.getTopicProperty().setValue(topic);
        messageDTO.getPayloadProperty().setValue(payload);
        messageDTO.getIsRetainedProperty().setValue(isRetained);
        messageDTO.getQosProperty().setValue(qos);
        messageDTO.getDateTimeProperty().setValue(dateTime);
        messageDTO.getMessageIdProperty().setValue(messageId);
        messageDTO.getMessageTypeProperty().setValue(messageType);
        messageDTO.getPublishStatusProperty().setValue(publishStatus);
        messageDTO.getExtraProperties().setValue(new ObservableMapWrapper<>(customFields));
        return messageDTO;
    }
}
