package org.correomqtt.gui.components;

import javafx.beans.property.SimpleStringProperty;
import javafx.beans.property.StringProperty;
import javafx.beans.value.ObservableValue;
import javafx.geometry.Insets;
import javafx.scene.control.Button;
import javafx.scene.control.Label;
import javafx.scene.paint.Paint;

public class IconLabel extends Label {

    private final StringProperty iconProperty = new SimpleStringProperty();
    private ThemedFontIcon themedFontIcon;

    public IconLabel() {
        super();
        iconProperty.addListener(this::iconChange);
    }

    private void iconChange(ObservableValue<? extends String> observable, String oldValue, String newValue) {
        themedFontIcon = new ThemedFontIcon();
        themedFontIcon.setIconLiteral(newValue);
        themedFontIcon.setIconSize(18);
        this.setGraphic(themedFontIcon);
        this.setPadding(new Insets(1,5,1,5));
        this.setHeight(28);
        this.setMinHeight(28);
    }

    public void setIcon(String icon) {
        this.iconProperty.set(icon);
    }

    public String getIcon() {
        return this.iconProperty.get();
    }

    public void setIconColor(Paint paint){
        this.themedFontIcon.setIconColor(paint);
    }
}
