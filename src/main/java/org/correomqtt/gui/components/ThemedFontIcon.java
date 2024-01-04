package org.correomqtt.gui.components;

import javafx.scene.paint.Paint;
import org.correomqtt.business.fileprovider.SettingsProvider;
import org.kordamp.ikonli.javafx.FontIcon;

public class ThemedFontIcon extends FontIcon {

    public ThemedFontIcon() {
        super();
        this.iconColorProperty().setValue(Paint.valueOf(SettingsProvider.getInstance().getIconModeCssClass()));
    }
}
