package org.correomqtt.gui.theme;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonValue;

import java.util.Arrays;

public enum IconMode {
    BLACK("black"),WHITE("white");

    private final String jsonValue;

    IconMode(String white) {
        this.jsonValue = white;
    }

    @Override
    public String toString() {
        return jsonValue;
    }

    @JsonValue
    public String getJsonValue() {
        return jsonValue;
    }

    @JsonCreator
    public static IconMode forValue(String iconMode) {

        return Arrays.stream(IconMode.values())
                .filter(im -> im.jsonValue.equalsIgnoreCase(iconMode))
                .findFirst()
                .orElse(BLACK);
    }
}
