package com.exxeta.correomqtt.gui.model;

import com.exxeta.correomqtt.business.model.MessageType;
import com.exxeta.correomqtt.business.model.PublishStatus;
import com.exxeta.correomqtt.business.model.Qos;
import com.exxeta.correomqtt.business.utils.MessageDateTimeFormatter;
import javafx.beans.Observable;
import javafx.beans.property.*;
import javafx.util.Callback;
import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Getter;

import java.time.LocalDateTime;
import java.util.HashMap;

@Getter
@AllArgsConstructor
@Builder
public class MessagePropertiesDTO implements Comparable<MessagePropertiesDTO> {

    private final StringProperty topicProperty;
    private final StringProperty payloadProperty;
    private final BooleanProperty isRetainedProperty;
    private final Property<Qos> qosProperty;
    private final Property<LocalDateTime> dateTimeProperty;
    private final Property<SubscriptionPropertiesDTO> subscriptionDTOProperty;
    private final StringProperty messageIdProperty;
    private final Property<MessageType> messageTypeProperty;
    private final Property<PublishStatus> publishStatusProperty;
    private final MapProperty<String, Object> extraProperties;

    public static Callback<MessagePropertiesDTO, Observable[]> extractor() {
        return (MessagePropertiesDTO m) -> new Observable[]{
                m.topicProperty,
                m.payloadProperty,
                m.isRetainedProperty,
                m.qosProperty,
                m.dateTimeProperty,
                m.subscriptionDTOProperty,
                m.messageIdProperty,
                m.messageTypeProperty,
                m.publishStatusProperty,
                m.extraProperties
        };
    }

    public String getMessageId() {
        return messageIdProperty.get();
    }

    public String getTopic() {
        return topicProperty.get();
    }

    public String getPayload() {
        return payloadProperty.get();
    }

    public void setPayload(String payload) {
        payloadProperty.set(payload);
    }

    public StringProperty getTopicProperty() {
        return topicProperty;
    }

    public Property<LocalDateTime> getDateTimeProperty() {
        return dateTimeProperty;
    }

    public boolean isRetained() {
        return isRetainedProperty.get();
    }

    public Qos getQos() {
        return qosProperty.getValue();
    }

    public SubscriptionPropertiesDTO getSubscription() {
        return subscriptionDTOProperty.getValue();
    }

    public PublishStatus getPublishStatus() {
        return publishStatusProperty.getValue();
    }

    public void setPublishStatus(PublishStatus publishStatus) {
        publishStatusProperty.setValue(publishStatus);
    }

    @MessageDateTimeFormatter
    public LocalDateTime getDateTime() {
        return dateTimeProperty.getValue();
    }

    @Override
    public String toString() {
        return "Message to " + getTopic() + " with QoS: " + getQos().ordinal() + " at " + getDateTime();
    }

    @Override
    public int hashCode() {
        return (getTopic() + getPayload() + isRetained() + getQos() + getDateTime()).hashCode();
    }

    @Override
    public boolean equals(Object o) {
        if (!(o instanceof MessagePropertiesDTO)) {
            return false;
        }

        MessagePropertiesDTO other = (MessagePropertiesDTO) o;

        return ((other.getTopic() != null && other.getTopic().equals(getTopic())) || (other.getTopic() == null && getTopic() == null)) &&
                ((other.getQos() != null && other.getQos().equals(getQos())) || (other.getQos() == null && getQos() == null)) &&
                ((other.getPayload() != null && other.getPayload().equals(getPayload())) || (other.getPayload() == null && getPayload() == null)) &&
                ((other.getDateTime() != null && other.getDateTime().equals(getDateTime())) || (other.getDateTime() == null && getDateTime() == null)) &&
                ((other.getExtraProperties() != null && other.getExtraProperties().equals(getExtraProperties())) || (other.getExtraProperties() == null && getExtraProperties() == null)) &&
                (other.isRetained() == isRetained());
    }

    @Override
    public int compareTo(MessagePropertiesDTO other) {
        if (other == null || other.getDateTime() == null) {
            return 1;
        }

        if (getDateTime() == null) {
            return -1;
        }

        return other.getDateTime().compareTo(other.getDateTime());
    }

    public MessageType getMessageType() {
        return messageTypeProperty.getValue();
    }

    @SuppressWarnings({"FieldCanBeLocal", "unused"})
    public static class MessagePropertiesDTOBuilder {

        private StringProperty topicProperty = new SimpleStringProperty();
        private StringProperty payloadProperty = new SimpleStringProperty();
        private BooleanProperty isRetainedProperty = new SimpleBooleanProperty(false);
        private Property<Qos> qosProperty = new SimpleObjectProperty<>();
        private Property<LocalDateTime> dateTimeProperty = new SimpleObjectProperty<>();
        private Property<SubscriptionPropertiesDTO> subscriptionDTOProperty = new SimpleObjectProperty<>();
        private StringProperty messageIdProperty = new SimpleStringProperty();
        private Property<MessageType> messageTypeProperty = new SimpleObjectProperty<>();
        private Property<PublishStatus> publishStatusProperty = new SimpleObjectProperty<>();
        private SimpleMapProperty<String, Object> extraProperties = new SimpleMapProperty<>();

        public MessagePropertiesDTOBuilder topic(String topic) {
            this.topicProperty.set(topic);
            return this;
        }

        public MessagePropertiesDTOBuilder payload(String payload) {
            this.payloadProperty.set(payload);
            return this;
        }

        public MessagePropertiesDTOBuilder isRetained(boolean isRetained) {
            this.isRetainedProperty.set(isRetained);
            return this;
        }

        public MessagePropertiesDTOBuilder qos(Qos qos) {
            this.qosProperty.setValue(qos);
            return this;
        }

        public MessagePropertiesDTOBuilder dateTime(LocalDateTime dateTime) {
            this.dateTimeProperty.setValue(dateTime);
            return this;
        }

        public MessagePropertiesDTOBuilder subscription(SubscriptionPropertiesDTO subscriptionPropertiesDTO) {
            this.subscriptionDTOProperty.setValue(subscriptionPropertiesDTO);
            return this;
        }

        public MessagePropertiesDTOBuilder messageId(String messageId) {
            this.messageIdProperty.set(messageId);
            return this;
        }

        public MessagePropertiesDTOBuilder messageType(MessageType messageType) {
            this.messageTypeProperty.setValue(messageType);
            return this;
        }

        public MessagePropertiesDTOBuilder publishStatus(PublishStatus publishStatus) {
            this.publishStatusProperty.setValue(publishStatus);
            return this;
        }

        private MessagePropertiesDTOBuilder extraProperties(HashMap<String, Object> extraProperties) {
            this.extraProperties.putAll(extraProperties);
            return this;
        }

        public MessagePropertiesDTO build() {
            return new MessagePropertiesDTO(topicProperty,
                    payloadProperty,
                    isRetainedProperty,
                    qosProperty,
                    dateTimeProperty,
                    subscriptionDTOProperty,
                    messageIdProperty,
                    messageTypeProperty,
                    publishStatusProperty,
                    extraProperties);
        }
    }
}
