package org.correomqtt.gui.controls;

import javafx.beans.property.SimpleStringProperty;
import javafx.beans.property.StringProperty;
import javafx.geometry.Insets;
import javafx.scene.control.Button;
import org.kordamp.ikonli.javafx.FontIcon;

public class IconButton extends Button {

    private final StringProperty iconProperty = new SimpleStringProperty();

    public IconButton() {
        super();
        iconProperty.addListener((ob, o, n) -> iconChange(n));
    }

    private void iconChange(String newValue) {
        FontIcon themedFontIcon = new FontIcon();
        themedFontIcon.setIconLiteral(newValue);
        themedFontIcon.setIconSize(18);
        this.setGraphic(themedFontIcon);
        this.setPadding(new Insets(1, 8, 1, 8));
        this.setHeight(28);
        this.setMinHeight(28);
    }

    public String getIcon() {
        return this.iconProperty.get();
    }

    public void setIcon(String icon) {
        this.iconProperty.set(icon);
    }
}
