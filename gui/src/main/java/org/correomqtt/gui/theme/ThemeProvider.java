package org.correomqtt.gui.theme;


import javafx.scene.paint.Color;
import org.correomqtt.core.model.GenericTranslatable;

public interface ThemeProvider extends GenericTranslatable {

    String getName();

    String getCss();

    IconMode getIconMode();

    default String getLabelTranslationKey() {
        return getName();
    }

    Color getBackgroundColor();
}
