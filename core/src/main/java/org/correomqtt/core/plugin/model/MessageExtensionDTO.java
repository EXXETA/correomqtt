package org.correomqtt.core.plugin.model;

import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Getter;
import lombok.Setter;
import org.correomqtt.core.model.MessageDTO;
import org.correomqtt.core.model.MessageType;
import org.correomqtt.core.model.PublishStatus;
import org.correomqtt.core.model.Qos;

import java.time.LocalDateTime;
import java.util.HashMap;

@Setter
@Getter
@Builder
@AllArgsConstructor
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
}
