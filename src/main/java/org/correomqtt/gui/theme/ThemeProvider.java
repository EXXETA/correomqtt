package org.correomqtt.gui.theme;


import org.correomqtt.business.model.GenericTranslatable;

public interface ThemeProvider extends GenericTranslatable {

    String getName();

    String getCss();

    IconMode getIconMode();

    default String getLabelTranslationKey() {
        return getName();
    }

}
