package org.correomqtt.business.model;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonInclude;
import lombok.*;

@Getter
@Setter
@AllArgsConstructor
@NoArgsConstructor
@Builder
@JsonInclude(JsonInclude.Include.NON_NULL)
@JsonIgnoreProperties(ignoreUnknown = true)
public class ConnectionUISettings {
    private double mainDividerPosition;
    private double publishDividerPosition;
    private boolean publishDetailActive;
    private double subscribeDividerPosition;
    private boolean subscribeDetailActive;

    public double getMainDividerPosition() {
        return mainDividerPosition;
    }

    public void setMainDividerPosition(double mainDividerPosition) {
        this.mainDividerPosition = mainDividerPosition;
    }

    public double getPublishDividerPosition() {
        return publishDividerPosition;
    }

    public void setPublishDividerPosition(double publishDividerPosition) {
        this.publishDividerPosition = publishDividerPosition;
    }

    public boolean isPublishDetailActive() {
        return publishDetailActive;
    }

    public void setPublishDetailActive(boolean publishDetailActive) {
        this.publishDetailActive = publishDetailActive;
    }

    public double getSubscribeDividerPosition() {
        return subscribeDividerPosition;
    }

    public void setSubscribeDividerPosition(double subscribeDividerPosition) {
        this.subscribeDividerPosition = subscribeDividerPosition;
    }

    public boolean isSubscribeDetailActive() {
        return subscribeDetailActive;
    }

    public void setSubscribeDetailActive(boolean subscribeDetailActive) {
        this.subscribeDetailActive = subscribeDetailActive;
    }
}
