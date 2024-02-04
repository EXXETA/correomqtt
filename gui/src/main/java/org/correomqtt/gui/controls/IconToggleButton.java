package org.correomqtt.gui.controls;

import javafx.beans.property.SimpleStringProperty;
import javafx.beans.property.StringProperty;
import javafx.geometry.Insets;
import javafx.scene.control.ToggleButton;

public class IconToggleButton extends ToggleButton {

    private final StringProperty iconProperty = new SimpleStringProperty();

    public IconToggleButton() {
        super();
        iconProperty.addListener((ob, o, n) -> iconChange(n));
    }

    private void iconChange(String newValue) {
        ThemedFontIcon themedFontIcon = new ThemedFontIcon();
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
