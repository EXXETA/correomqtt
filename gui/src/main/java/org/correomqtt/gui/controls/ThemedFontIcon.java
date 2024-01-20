package org.correomqtt.gui.controls;

import javafx.scene.paint.Paint;
import org.correomqtt.business.settings.SettingsProvider;
import org.kordamp.ikonli.javafx.FontIcon;

public class ThemedFontIcon extends FontIcon {

    public ThemedFontIcon() {
        super();
        this.iconColorProperty().setValue(Paint.valueOf(SettingsProvider.getInstance().getIconModeCssClass()));
    }

    public ThemedFontIcon(String iconLiteral) {
        super();
        this.setIconLiteral(iconLiteral);
        this.iconColorProperty().setValue(Paint.valueOf(SettingsProvider.getInstance().getIconModeCssClass()));
    }


    public ThemedFontIcon(String iconLiteral, Paint color) {
        super();
        this.setIconLiteral(iconLiteral);
        this.setIconColor(color);
        this.iconColorProperty().setValue(Paint.valueOf(SettingsProvider.getInstance().getIconModeCssClass()));
    }

    public ThemedFontIcon(String iconLiteral, Integer iconSize) {
        super();
        this.setIconLiteral(iconLiteral);
        this.setIconSize(iconSize);
        this.iconColorProperty().setValue(Paint.valueOf(SettingsProvider.getInstance().getIconModeCssClass()));
    }

}
