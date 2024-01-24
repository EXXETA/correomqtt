package org.correomqtt.gui.controls;

import javafx.beans.property.BooleanProperty;
import javafx.beans.property.BooleanPropertyBase;
import javafx.beans.property.SimpleStringProperty;
import javafx.beans.property.StringProperty;
import javafx.event.ActionEvent;
import javafx.scene.Node;
import javafx.scene.control.MenuItem;

public class IconCheckMenuItem extends MenuItem {

    private static final String CHECK_ICON = "mdi-checkbox-marked";
    private static final String UNCHECK_ICON = "mdi-checkbox-blank-outline";

    private final StringProperty iconProperty = new SimpleStringProperty();
    private BooleanProperty selected;


    public IconCheckMenuItem() {
        this(null, null);
    }


    public IconCheckMenuItem(String text, Node graphic) {
        super(text, graphic);
        iconProperty.addListener((ob, o, n) -> iconChange(n));
        addEventHandler(ActionEvent.ACTION, this::onActionHandler);
        setIcon(UNCHECK_ICON);
    }

    private void iconChange(String newValue) {
        ThemedFontIcon themedFontIcon = new ThemedFontIcon();
        themedFontIcon.setIconLiteral(newValue);
        themedFontIcon.setIconSize(18);
        this.setGraphic(themedFontIcon);
    }

    private void onActionHandler(ActionEvent actionEvent) {
        setSelected(!isSelected());
    }

    public final boolean isSelected() {
        return selected != null && selected.get();
    }

    public final void setSelected(boolean value) {
        selectedProperty().set(value);
    }

    public final BooleanProperty selectedProperty() {
        if (selected == null) {
            selected = new BooleanPropertyBase() {
                @Override
                protected void invalidated() {
                    get();

                    if (isSelected()) {
                        setIcon(CHECK_ICON);
                    } else {
                        setIcon(UNCHECK_ICON);
                    }
                }

                @Override
                public Object getBean() {
                    return IconCheckMenuItem.this;
                }

                @Override
                public String getName() {
                    return "selected";
                }
            };
        }
        return selected;
    }

    public IconCheckMenuItem(String text) {
        this(text, null);
    }

    public String getIcon() {
        return this.iconProperty.get();
    }

    public void setIcon(String icon) {
        this.iconProperty.set(icon);
    }

}
