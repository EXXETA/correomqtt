package org.correomqtt.gui.theme;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonValue;

import java.util.Arrays;

public enum IconMode {
    BLACK("black"),WHITE("white");

    private final String iconMode;

    IconMode(String white) {
        this.iconMode = white;
    }

    @Override
    public String toString() {
        return iconMode;
    }

    @JsonValue
    public String getIconMode() {
        return iconMode;
    }

    @JsonCreator
    public static IconMode forValue(String iconMode) {

        return Arrays.stream(IconMode.values())
                .filter(im -> im.iconMode.equalsIgnoreCase(iconMode))
                .findFirst()
                .orElse(BLACK);
    }
}
