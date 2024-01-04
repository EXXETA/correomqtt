package org.correomqtt.gui.components;

import javafx.beans.property.SimpleStringProperty;
import javafx.beans.property.StringProperty;
import javafx.beans.value.ObservableValue;
import javafx.scene.control.MenuButton;
import javafx.scene.control.MenuItem;

public class IconMenuItem extends MenuItem {

    private final StringProperty iconProperty = new SimpleStringProperty();

    public IconMenuItem(){
        super();
        iconProperty.addListener(this::iconChange);
    }

    private void iconChange(ObservableValue<? extends String> observable, String oldValue, String newValue) {
        ThemedFontIcon themedFontIcon = new ThemedFontIcon();
        themedFontIcon.setIconLiteral(newValue);
        themedFontIcon.setIconSize(18);
        this.setGraphic(themedFontIcon);
    }

    public void setIcon(String icon){
        this.iconProperty.set(icon);
    }

    public String getIcon(){
       return this.iconProperty.get();
    }
}
