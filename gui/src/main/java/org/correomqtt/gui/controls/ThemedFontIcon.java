package org.correomqtt.gui.controls;

import javafx.scene.paint.Paint;
import org.correomqtt.CorreoAppComponent;
import org.correomqtt.FxApplication;
import org.kordamp.ikonli.javafx.FontIcon;

public class ThemedFontIcon extends FontIcon {

    public ThemedFontIcon() {
        super();
        setupIconColor();
    }

    private void setupIconColor() {
        CorreoAppComponent component = FxApplication.getAppComponent();

        String iconClass;
        if (component == null) {
            iconClass = "white";
        } else {
            iconClass = component.themeManager().getIconModeCssClass();
        }
        if (iconClass != null && !iconClass.isEmpty()) {
            this.iconColorProperty().setValue(Paint.valueOf(iconClass));
        }
    }

    public ThemedFontIcon(String iconLiteral) {
        super();
        this.setIconLiteral(iconLiteral);
        setupIconColor();
    }

    public ThemedFontIcon(String iconLiteral, Paint color) {
        super();
        this.setIconLiteral(iconLiteral);
        this.setIconColor(color);
    }

    public ThemedFontIcon(String iconLiteral, Integer iconSize) {
        super();
        this.setIconLiteral(iconLiteral);
        this.setIconSize(iconSize);
        setupIconColor();
    }

    public ThemedFontIcon(String iconLiteral, Paint color, Integer iconSize) {
        super();
        this.setIconLiteral(iconLiteral);
        this.setIconSize(iconSize);
        this.setIconColor(color);
    }

}
