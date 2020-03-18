package org.correomqtt.gui.utils;

import javafx.stage.Window;

import java.util.Map;

public class WindowHelper {

    private WindowHelper() {
        // private constructor
    }

    public static Window getWindow(Map<Object, Object> properties) {
        return Window.getWindows().stream()
                     .filter(w -> properties.entrySet().stream()
                                            .allMatch(p -> {
                                                Map<Object, Object> windowProperties = w.getProperties();
                                                Object windowValue = windowProperties.get(p.getKey());
                                                return (windowValue != null) && windowValue.equals(p.getValue());
                                            })
                     )
                     .findFirst()
                     .orElse(null);
    }

    public static boolean focusWindowIfAlreadyThere(Map<Object, Object> properties) {
        Window window = getWindow(properties);

        if (window != null) {
            window.requestFocus();
            return true;
        }

        return false;
    }
}
