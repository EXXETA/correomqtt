package org.correomqtt.gui.contextmenu;

import javafx.scene.control.ContextMenu;
import org.correomqtt.core.settings.SettingsManager;

public abstract class BaseObjectContextMenu<O, D extends BaseObjectContextMenuDelegate> extends ContextMenu {

    protected final SettingsManager settingsManager;
    protected final D delegate;
    protected O dto;

    protected BaseObjectContextMenu(SettingsManager settingsManager, D delegate) {
        super();
        this.settingsManager = settingsManager;
        this.delegate = delegate;
        initializeItems();
        setVisibilityForObjectItems(false);
    }

    public void setObject(O dto) {
        this.dto = dto;
        setVisibilityForObjectItems(dto != null);
    }

    protected void initializeItems() {
        // nothing to do
    }

    protected void setVisibilityForObjectItems(boolean visible) {
        // nothing to do
    }

    String getClassName() {
        return Thread.currentThread().getStackTrace()[2].getClassName();
    }
}
