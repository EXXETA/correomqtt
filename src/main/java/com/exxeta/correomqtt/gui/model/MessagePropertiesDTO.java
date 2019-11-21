package com.exxeta.correomqtt.gui.model;

import com.exxeta.correomqtt.business.model.MessageType;
import com.exxeta.correomqtt.business.model.PublishStatus;
import com.exxeta.correomqtt.business.model.Qos;
import com.exxeta.correomqtt.business.services.ConfigService;
import com.exxeta.correomqtt.business.utils.MessageDateTimeFormatter;
import javafx.beans.Observable;
import javafx.beans.property.BooleanProperty;
import javafx.beans.property.Property;
import javafx.beans.property.SimpleBooleanProperty;
import javafx.beans.property.SimpleObjectProperty;
import javafx.beans.property.SimpleStringProperty;
import javafx.beans.property.StringProperty;
import javafx.util.Callback;
import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Getter;

import java.time.LocalDateTime;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

@Getter
@AllArgsConstructor
@Builder
public class MessagePropertiesDTO implements Comparable<MessagePropertiesDTO> {

    private static final Pattern MESSAGE_ID_ANSWER_EXPECTED_PATTERN = Pattern
            .compile("^([0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[0-9a-f]{4}-[0-9a-f]{12})([0-1]{1})", Pattern.CASE_INSENSITIVE);

    private final StringProperty topicProperty;
    private final StringProperty payloadProperty;
    private final BooleanProperty isRetainedProperty;
    private final Property<Qos> qosProperty;
    private final Property<LocalDateTime> dateTimeProperty;
    private final Property<SubscriptionPropertiesDTO> subscriptionDTOProperty;
    private final StringProperty specialMessageIdProperty;
    private final BooleanProperty answerExpectedProperty;
    private final StringProperty messageIdProperty;
    private final Property<MessageType> messageTypeProperty;
    private final Property<PublishStatus> publishStatusProperty;

    public static Callback<MessagePropertiesDTO, Observable[]> extractor() {
        return (MessagePropertiesDTO m) -> new Observable[]{
                m.topicProperty,
                m.payloadProperty,
                m.isRetainedProperty,
                m.qosProperty,
                m.dateTimeProperty,
                m.subscriptionDTOProperty,
                m.specialMessageIdProperty,
                m.answerExpectedProperty,
                m.messageIdProperty,
                m.messageTypeProperty,
                m.publishStatusProperty
        };
    }

    public String getMessageId(){
        return messageIdProperty.get();
    }

    public String getTopic() {
        return topicProperty.get();
    }

    public String getPayload() {
        return payloadProperty.get();
    }

    public void setPayload(String payload) {
        if (ConfigService.getInstance().getSettings().isExtraFeatures() && getPayload() != null) {
            Matcher m = MESSAGE_ID_ANSWER_EXPECTED_PATTERN.matcher(getPayload());
            if (m.find()) {
                specialMessageIdProperty.set(m.group(1));
                answerExpectedProperty.set("1".equals(m.group(2)));
                payloadProperty.set(payload.substring(37));
            } else {
                payloadProperty.set(payload);
            }
        } else {
            payloadProperty.set(payload);
        }
    }

    public StringProperty getTopicProperty() {
        return topicProperty;
    }

    public Property<LocalDateTime> getDateTimeProperty() {
        return dateTimeProperty;
    }

    public String getSpecialMessageId() {
        return specialMessageIdProperty.get();
    }

    public boolean isAnswerExpected() {
        return answerExpectedProperty.get();
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

    public void setPublishStatus (PublishStatus publishStatus) {
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
                ((other.getSpecialMessageId() != null && other.getSpecialMessageId().equals(getSpecialMessageId())) || (other.getSpecialMessageId() == null && getSpecialMessageId() == null)) &&
                (other.isRetained() == isRetained()) &&
                (other.isAnswerExpected() == isAnswerExpected());
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
        private StringProperty specialMessageIdProperty = new SimpleStringProperty();
        private BooleanProperty answerExpectedProperty = new SimpleBooleanProperty(false);
        private StringProperty messageIdProperty = new SimpleStringProperty();
        private Property<MessageType> messageTypeProperty = new SimpleObjectProperty<>();
        private Property<PublishStatus> publishStatusProperty = new SimpleObjectProperty<>();

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

        public MessagePropertiesDTOBuilder specialMessageId(String specialMessageId) {
            this.specialMessageIdProperty.set(specialMessageId);
            return this;
        }

        public MessagePropertiesDTOBuilder answerExpected(boolean answerExpected) {
            this.answerExpectedProperty.set(answerExpected);
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

        public MessagePropertiesDTO build() {
            return new MessagePropertiesDTO(topicProperty,
                                            payloadProperty,
                                            isRetainedProperty,
                                            qosProperty,
                                            dateTimeProperty,
                                            subscriptionDTOProperty,
                                            specialMessageIdProperty,
                                            answerExpectedProperty,
                                            messageIdProperty,
                                            messageTypeProperty,
                                            publishStatusProperty);
        }
    }
}