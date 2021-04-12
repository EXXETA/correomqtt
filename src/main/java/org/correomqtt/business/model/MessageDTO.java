package org.correomqtt.business.model;

import org.correomqtt.business.utils.CorreoCharsetDecoder;
import org.correomqtt.business.utils.MessageDateTimeFormatter;
import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonInclude;
import com.hivemq.client.mqtt.mqtt3.message.publish.Mqtt3Publish;
import com.hivemq.client.mqtt.mqtt5.message.publish.Mqtt5Publish;
import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Getter;
import lombok.NoArgsConstructor;
import lombok.Setter;

import java.time.LocalDateTime;
import java.time.ZoneOffset;
import java.util.Objects;
import java.util.UUID;

@Getter
@Setter
@Builder
@NoArgsConstructor
@AllArgsConstructor
@JsonInclude(JsonInclude.Include.NON_NULL)
@JsonIgnoreProperties(ignoreUnknown = true)
public class MessageDTO implements Comparable<MessageDTO> {
    private String topic;
    private String payload;
    private boolean isRetained;
    private boolean isFavorited;
    private Qos qos;
    @MessageDateTimeFormatter
    private LocalDateTime dateTime;
    private String messageId;
    private MessageType messageType;
    private PublishStatus publishStatus;

    public MessageDTO(Mqtt3Publish mqtt3Publish) {
        setTopic(mqtt3Publish.getTopic().toString());
        setQos(Qos.valueOf(mqtt3Publish.getQos()));
        setRetained(mqtt3Publish.isRetain());
        setPayload(CorreoCharsetDecoder.decode(mqtt3Publish.getPayloadAsBytes()));
        setDateTime(LocalDateTime.now(ZoneOffset.UTC));
        setMessageId(UUID.randomUUID().toString());
        setMessageType(MessageType.INCOMING);
    }

    public MessageDTO(Mqtt5Publish mqtt5Publish) {
        setTopic(mqtt5Publish.getTopic().toString());
        setQos(Qos.valueOf(mqtt5Publish.getQos()));
        setRetained(mqtt5Publish.isRetain());
        setPayload(CorreoCharsetDecoder.decode(mqtt5Publish.getPayloadAsBytes()));
        setDateTime(LocalDateTime.now(ZoneOffset.UTC));
        setMessageId(UUID.randomUUID().toString());
        setMessageType(MessageType.INCOMING);
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (o == null || getClass() != o.getClass()) return false;
        MessageDTO that = (MessageDTO) o;
        return isRetained() == that.isRetained() &&
                getTopic().equals(that.getTopic()) &&
                Objects.equals(getPayload(), that.getPayload()) &&
                getQos() == that.getQos() &&
                Objects.equals(getDateTime(), that.getDateTime());
    }

    @Override
    public int hashCode() {
        return Objects.hash(getTopic(), getPayload(), isRetained(), getQos(), getDateTime());
    }

    @Override
    public int compareTo(MessageDTO o) {
        if ((o == null || o.getDateTime() == null)) {
            return (getDateTime() == null) ? 0 : 1;
        }

        return o.getDateTime().compareTo(getDateTime());
    }
}
