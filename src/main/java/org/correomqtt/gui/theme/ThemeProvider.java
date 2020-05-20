package org.correomqtt.gui.theme;

import org.correomqtt.gui.model.GenericCellModel;

public interface ThemeProvider extends GenericCellModel {

    String getName();

    String getCss();

    IconMode getIconMode();

    default String getLabelTranslationKey() {
        return getName();
    }

}
