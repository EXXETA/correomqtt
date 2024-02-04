package org.correomqtt.gui.controls;

import javafx.beans.property.IntegerProperty;
import javafx.beans.property.ObjectProperty;
import javafx.beans.property.SimpleIntegerProperty;
import javafx.beans.property.SimpleObjectProperty;
import javafx.beans.property.SimpleStringProperty;
import javafx.beans.property.StringProperty;
import javafx.beans.value.ObservableValue;
import javafx.scene.control.Label;
import javafx.scene.paint.Paint;

public class IconLabel extends Label {

    private final StringProperty iconProperty = new SimpleStringProperty();

    private final ObjectProperty<Paint> iconColorProperty = new SimpleObjectProperty<>();
    private final IntegerProperty iconSizeProperty = new SimpleIntegerProperty(18);
    private ThemedFontIcon themedFontIcon;

    public IconLabel() {
        super();
        iconProperty.addListener(this::iconChange);
        iconSizeProperty.addListener(this::iconChange);
        iconColorProperty.addListener(this::iconChange);
    }

    private void iconChange(ObservableValue<?> observable, Object oldValue, Object newValue) {
        themedFontIcon = new ThemedFontIcon();
        themedFontIcon.setIconLiteral(iconProperty.get());
        themedFontIcon.setIconSize(iconSizeProperty.get());
        if (iconColorProperty.get() != null) {
            themedFontIcon.setIconColor(iconColorProperty.get());
        }
        this.setGraphic(themedFontIcon);
    }

    public void setIcon(String icon) {
        this.iconProperty.set(icon);
    }

    public String getIcon() {
        return this.iconProperty.get();
    }

    public void setIconColor(Paint paint) {
        this.themedFontIcon.setIconColor(paint);
    }

    public Paint getIconColor() {
        return this.themedFontIcon.getIconColor();
    }

    public void setIconSize(Integer iconSize) {
        this.iconSizeProperty.set(iconSize);
    }

    public Integer getIconSize() {
        return this.iconSizeProperty.get();
    }

}
