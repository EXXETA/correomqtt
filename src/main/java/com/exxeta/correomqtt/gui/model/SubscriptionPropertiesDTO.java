package com.exxeta.correomqtt.gui.model;

import com.exxeta.correomqtt.business.model.Qos;
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

@Getter
@AllArgsConstructor
@Builder
public class SubscriptionPropertiesDTO {

    private final StringProperty topicProperty;
    private final Property<Qos> qosProperty;
    private final BooleanProperty filteredProperty;
    private final BooleanProperty hiddenProperty;

    public static Callback<SubscriptionPropertiesDTO, Observable[]> extractor() {
        return (SubscriptionPropertiesDTO m) -> new Observable[]{
                m.topicProperty,
                m.qosProperty,
                m.filteredProperty,
                m.hiddenProperty
        };
    }

    public String getTopic() {
        return topicProperty.getValue();
    }

    public void setTopic(String topic) {
        topicProperty.setValue(topic);
    }

    public Qos getQos() {
        return qosProperty.getValue();
    }

    public void setQos(Qos qos) {
        qosProperty.setValue(qos);
    }

    public boolean isFiltered() {
        return filteredProperty.get();
    }

    public void setFiltered(boolean filtered) {
        this.filteredProperty.set(filtered);
    }

    public boolean isHidden() {
        return hiddenProperty.get();
    }

    public void setHidden(boolean hidden) {
        this.filteredProperty.set(hidden);
    }

    @Override
    public int hashCode() {
        return topicProperty.hashCode() + qosProperty.hashCode() + filteredProperty.hashCode() + hiddenProperty.hashCode();
    }

    @Override
    public boolean equals(Object o) {
        if (!(o instanceof SubscriptionPropertiesDTO)) {
            return false;
        }

        SubscriptionPropertiesDTO other = (SubscriptionPropertiesDTO) o;

        return other.topicProperty.equals(topicProperty) &&
                other.qosProperty.equals(qosProperty) &&
                other.filteredProperty.equals(filteredProperty);
    }

    @SuppressWarnings({"FieldCanBeLocal", "unused"})
    public static class SubscriptionPropertiesDTOBuilder {

        private StringProperty topicProperty = new SimpleStringProperty();
        private Property<Qos> qosProperty = new SimpleObjectProperty<>();
        private BooleanProperty filteredProperty = new SimpleBooleanProperty(true);
        private BooleanProperty hiddenProperty = new SimpleBooleanProperty(false);

        public SubscriptionPropertiesDTOBuilder topic(String topic) {
            this.topicProperty.set(topic);
            return this;
        }

        public SubscriptionPropertiesDTOBuilder qos(Qos qos) {
            this.qosProperty.setValue(qos);
            return this;
        }

        public SubscriptionPropertiesDTOBuilder filtered(boolean filtered) {
            this.filteredProperty.set(filtered);
            return this;
        }

        public SubscriptionPropertiesDTOBuilder hidden(boolean hidden) {
            this.hiddenProperty.set(hidden);
            return this;
        }

        public SubscriptionPropertiesDTO build() {
            return new SubscriptionPropertiesDTO(topicProperty,
                                                 qosProperty,
                                                 filteredProperty,
                                                 hiddenProperty);
        }
    }

}
