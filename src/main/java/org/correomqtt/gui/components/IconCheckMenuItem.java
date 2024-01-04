package org.correomqtt.gui.components;

import javafx.beans.Observable;
import javafx.beans.property.SimpleStringProperty;
import javafx.beans.property.StringProperty;
import javafx.beans.value.ObservableValue;
import javafx.scene.control.CheckMenuItem;

public class IconCheckMenuItem extends CheckMenuItem {

    private final StringProperty iconProperty = new SimpleStringProperty();
    private ThemedFontIcon themedFontIcon;

    public IconCheckMenuItem(){
        super();
        iconProperty.addListener(this::iconChange);
        selectedProperty().addListener(this::selectChange);
    }

    private void selectChange(Observable observable) {
       iconProperty.setValue("mdi-eye");

    }

    private void iconChange(ObservableValue<? extends String> observable, String oldValue, String newValue) {
        themedFontIcon = new ThemedFontIcon();
        themedFontIcon.setIconLiteral(newValue);
        themedFontIcon.setIconSize(18);
        this.setGraphic(themedFontIcon);
        this.set
    }


    public void setIcon(String icon){
        this.iconProperty.set(icon);
    }

    public String getIcon(){
       return this.iconProperty.get();
    }
}
