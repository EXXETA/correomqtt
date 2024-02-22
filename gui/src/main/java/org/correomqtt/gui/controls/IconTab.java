package org.correomqtt.gui.controls;

import javafx.beans.Observable;
import javafx.beans.property.IntegerProperty;
import javafx.beans.property.SimpleIntegerProperty;
import javafx.beans.property.SimpleStringProperty;
import javafx.beans.property.StringProperty;
import javafx.scene.control.Tab;
import org.kordamp.ikonli.javafx.FontIcon;

public class IconTab extends Tab {

    private final StringProperty iconProperty = new SimpleStringProperty();
    private final IntegerProperty iconSizeProperty = new SimpleIntegerProperty(18);
    private final FontIcon fontIcon;

    public IconTab() {
        super();
        fontIcon = new FontIcon();
        setGraphic(fontIcon);
        iconProperty.addListener(this::iconChange);
        iconSizeProperty.addListener(this::iconSizeChange);
    }

    private void iconChange(Observable observable) {
        fontIcon.setIconLiteral(iconProperty.get());
    }

    private void iconSizeChange(Observable observable) {
        fontIcon.setIconSize(iconSizeProperty.get());
    }

    public String getIcon() {
        return this.iconProperty.get();
    }

    public void setIcon(String icon) {
        this.iconProperty.set(icon);
    }

    public Integer getIconSize() {
        return this.iconSizeProperty.get();
    }

    public void setIconSize(Integer iconSize) {
        this.iconSizeProperty.set(iconSize);
    }
}
