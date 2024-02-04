package org.correomqtt.gui.controls;

import javafx.beans.property.SimpleStringProperty;
import javafx.beans.property.StringProperty;
import javafx.scene.control.Tab;
import javafx.scene.paint.Paint;

public class IconTab extends Tab {

    private final StringProperty iconProperty = new SimpleStringProperty();
    private ThemedFontIcon themedFontIcon;

    public IconTab() {
        super();
        iconProperty.addListener((ob, o, n) -> iconChange(n));
    }

    private void iconChange(String newValue) {
        themedFontIcon = new ThemedFontIcon();
        themedFontIcon.setIconLiteral(newValue);
        themedFontIcon.setIconSize(18);
        this.setGraphic(themedFontIcon);
    }

    public String getIcon() {
        return this.iconProperty.get();
    }

    public void setIcon(String icon) {
        this.iconProperty.set(icon);
    }

    public void setIconColor(Paint paint) {
        this.themedFontIcon.setIconColor(paint);
    }
}
