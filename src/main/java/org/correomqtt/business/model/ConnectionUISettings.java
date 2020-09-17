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
    private boolean showSubscribe;
    private boolean showPublish;
    private double mainDividerPosition;
    private double publishDividerPosition;
    private double publishDetailDividerPosition;
    private boolean publishDetailActive;
    private double subscribeDividerPosition;
    private double subscribeDetailDividerPosition;
    private boolean subscribeDetailActive;

    public boolean isShowSubscribe() {
        return showSubscribe;
    }

    public void setShowSubscribe(boolean showSubscribe) {
        this.showSubscribe = showSubscribe;
    }

    public boolean isShowPublish() {
        return showPublish;
    }

    public void setShowPublish(boolean showPublish) {
        this.showPublish = showPublish;
    }

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

    public double getPublishDetailDividerPosition() {
        return publishDetailDividerPosition;
    }

    public void setPublishDetailDividerPosition(double publishDetailDividerPosition) {
        this.publishDetailDividerPosition = publishDetailDividerPosition;
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

    public double getSubscribeDetailDividerPosition() {
        return subscribeDetailDividerPosition;
    }

    public void setSubscribeDetailDividerPosition(double subscribeDetailDividerPosition) {
        this.subscribeDetailDividerPosition = subscribeDetailDividerPosition;
    }

    public boolean isSubscribeDetailActive() {
        return subscribeDetailActive;
    }

    public void setSubscribeDetailActive(boolean subscribeDetailActive) {
        this.subscribeDetailActive = subscribeDetailActive;
    }
}
