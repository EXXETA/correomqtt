package org.correomqtt.gui.utils;

import javafx.scene.paint.Color;

public class LuminanceColorHelper {

    private LuminanceColorHelper() {
        // private constructor
    }

    private static Color hexToRgb(String color) {
        return Color.valueOf(color);
    }

    public static String getForegroundColor(String backgroundColor) {
        Color rgb = hexToRgb(backgroundColor);
        double red = rgb.getRed();
        double green = rgb.getGreen();
        double blue = rgb.getBlue();

        //For calculation spec see: https://www.w3.org/TR/WCAG20/#relativeluminancedef
        double luminance = red * 0.2126 + green * 0.7152 + blue * 0.0722;

        //Threshold is taken arbitrary and depends on taste
        if (luminance > 0.7) {
            return "black";
        } else {
            return "white";
        }
    }
}
