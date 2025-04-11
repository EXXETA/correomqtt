package org.correomqtt.plugins.systopic.model;

import org.correomqtt.core.utils.MessageDateTimeFormatter;
import javafx.beans.Observable;
import javafx.beans.property.Property;
import javafx.beans.property.SimpleObjectProperty;
import javafx.beans.property.SimpleStringProperty;
import javafx.beans.property.StringProperty;
import javafx.util.Callback;
import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Getter;

import java.time.LocalDateTime;

@Getter
@AllArgsConstructor
@Builder
public class SysTopicPropertiesDTO implements Comparable<SysTopicPropertiesDTO> {

    private final StringProperty topicProperty;
    private final StringProperty payloadProperty;
    private final Property<LocalDateTime> dateTimeProperty;
    private final SysTopic sysTopic;
    private String min1;
    private String min5;
    private String min15;

    public static Callback<SysTopicPropertiesDTO, Observable[]> extractor() {
        return (SysTopicPropertiesDTO m) -> new Observable[]{
                m.topicProperty,
                m.payloadProperty,
                m.dateTimeProperty
        };
    }


    public void setMin1(String min1) {
        this.min1 = min1;
    }

    public void setMin5(String min5) {
        this.min5 = min5;
    }

    public void setMin15(String min15) {
        this.min15 = min15;
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

    @MessageDateTimeFormatter
    public LocalDateTime getDateTime() {
        return dateTimeProperty.getValue();
    }

    @Override
    public String toString() {
        return "SysTopic to " + getTopic() + " at " + getDateTime();
    }

    @Override
    public int hashCode() {
        return getTopic().hashCode();
    }

    @Override
    public boolean equals(Object o) {
        if (!(o instanceof SysTopicPropertiesDTO)) {
            return false;
        }

        SysTopicPropertiesDTO other = (SysTopicPropertiesDTO) o;

        return ((other.getTopic() != null && other.getTopic().equals(getTopic())) || (other.getTopic() == null && getTopic() == null));
    }

    @Override
    public int compareTo(SysTopicPropertiesDTO other) {
        if (other == null || other.getTopic() == null) {
            return 1;
        }

        if (getTopic() == null) {
            return -1;
        }

        return other.getTopic().compareTo(other.getTopic());
    }

    public String getAggregatedPayload() {
        if (!getMin1().isEmpty() || !getMin5().isEmpty() || !getMin15().isEmpty()) {
            return (getMin1().isEmpty() ? "-" : getMin1()) + " / " +
                    (getMin5().isEmpty() ? "-" : getMin5()) + " / " +
                    (getMin15().isEmpty() ? "-" : getMin15());
        } else {
            return getPayload(); //TODO UTF-8
        }
    }

    @SuppressWarnings({"FieldCanBeLocal", "unused"})
    public static class SysTopicPropertiesDTOBuilder {

        private StringProperty topicProperty = new SimpleStringProperty();
        private StringProperty payloadProperty = new SimpleStringProperty();
        private Property<LocalDateTime> dateTimeProperty = new SimpleObjectProperty<>();

        public SysTopicPropertiesDTOBuilder topic(String topic) {
            this.topicProperty.set(topic);
            return this;
        }

        public SysTopicPropertiesDTOBuilder payload(String payload) {
            this.payloadProperty.set(payload);
            return this;
        }

        public SysTopicPropertiesDTOBuilder dateTime(LocalDateTime dateTime) {
            this.dateTimeProperty.setValue(dateTime);
            return this;
        }

        public SysTopicPropertiesDTOBuilder sysTopic(SysTopic sysTopic) {
            this.sysTopic = sysTopic;
            return this;
        }

        public SysTopicPropertiesDTO build() {
            return new SysTopicPropertiesDTO(topicProperty,
                                             payloadProperty,
                                             dateTimeProperty,
                                             sysTopic,
                                             "", "", "");
        }
    }
}