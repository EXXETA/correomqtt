package org.correomqtt.gui.controls;

import javafx.beans.property.SimpleStringProperty;
import javafx.beans.property.StringProperty;
import javafx.scene.control.MenuItem;
import org.kordamp.ikonli.javafx.FontIcon;

public class IconMenuItem extends MenuItem {

    private final StringProperty iconProperty = new SimpleStringProperty();

    public IconMenuItem() {
        super();
        iconProperty.addListener((ob, o, n) -> iconChange(n));
    }

    private void iconChange(String newValue) {
        FontIcon fontIcon = new FontIcon();
        fontIcon.setIconLiteral(newValue);
        fontIcon.setIconSize(18);
        this.setGraphic(fontIcon);
    }

    public IconMenuItem(String text) {
        super(text);
        iconProperty.addListener((ob, o, n) -> iconChange(n));
    }

    public String getIcon() {
        return this.iconProperty.get();
    }

    public void setIcon(String icon) {
        this.iconProperty.set(icon);
    }
}
