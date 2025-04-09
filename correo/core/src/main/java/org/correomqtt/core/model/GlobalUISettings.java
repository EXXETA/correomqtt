package org.correomqtt.core.model;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonInclude;
import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Getter;
import lombok.NoArgsConstructor;
import lombok.Setter;

@Getter
@Setter
@AllArgsConstructor
@NoArgsConstructor
@Builder
@JsonInclude(JsonInclude.Include.NON_NULL)
@JsonIgnoreProperties(ignoreUnknown = true)
public class GlobalUISettings {
    private double windowPositionX;
    private double windowPositionY;
    private double windowWidth;
    private double windowHeight;

    public double getWindowPositionX() {
        return windowPositionX;
    }

    public void setWindowPositionX(double windowPositionX) {
        this.windowPositionX = windowPositionX;
    }

    public double getWindowPositionY() {
        return windowPositionY;
    }

    public void setWindowPositionY(double windowPositionY) {
        this.windowPositionY = windowPositionY;
    }

    public double getWindowWidth() {
        return windowWidth;
    }

    public void setWindowWidth(double windowWidth) {
        this.windowWidth = windowWidth;
    }

    public double getWindowHeight() {
        return windowHeight;
    }

    public void setWindowHeight(double windowHeight) {
        this.windowHeight = windowHeight;
    }
}
