package org.correomqtt.gui.views.scripting;

import dagger.assisted.Assisted;
import dagger.assisted.AssistedInject;
import javafx.scene.control.MenuItem;
import javafx.scene.control.SeparatorMenuItem;
import org.correomqtt.core.settings.SettingsProvider;
import org.correomqtt.gui.contextmenu.BaseObjectContextMenu;
import org.correomqtt.gui.controls.IconMenuItem;
import org.correomqtt.gui.controls.ThemedFontIcon;

import java.util.ResourceBundle;

public class ScriptContextMenu extends BaseObjectContextMenu<ScriptFilePropertiesDTO, ScriptContextMenuDelegate> {

    private final SettingsProvider settingsProvider;
    private MenuItem renameItem;
    private MenuItem deleteItem;
    private MenuItem runItem;
    private SeparatorMenuItem separator1;

    @AssistedInject
    public ScriptContextMenu(SettingsProvider settingsProvider,
                             @Assisted ScriptContextMenuDelegate delegate) {
        super(delegate);
        this.settingsProvider = settingsProvider;
        ResourceBundle resources = ResourceBundle.getBundle("org.correomqtt.i18n", settingsProvider.getSettings().getCurrentLocale());
    }

    @Override
    protected void initializeItems() {
//TODO translateion
        super.initializeItems();

        runItem = new IconMenuItem("Run");
        runItem.setGraphic(new ThemedFontIcon("mdi-play"));
        runItem.setOnAction(event -> delegate.runScript(dto));

        renameItem = new IconMenuItem("Rename");
        renameItem.setGraphic(new ThemedFontIcon("mdi-rename"));
        renameItem.setOnAction(event -> delegate.renameScript(dto));

        deleteItem = new IconMenuItem("Delete");
        deleteItem.setGraphic(new ThemedFontIcon("mdi-trash-can"));
        deleteItem.setOnAction(event -> delegate.deleteScript(dto));

        separator1 = new SeparatorMenuItem();

        this.getItems().addAll(runItem,
                separator1,
                renameItem,
                deleteItem
        );
    }

    @Override
    protected void setVisibilityForObjectItems(boolean visible) {
        super.setVisibilityForObjectItems(visible);
        renameItem.setVisible(visible);
        deleteItem.setVisible(visible);
        runItem.setVisible(visible);
        separator1.setVisible(visible);
    }


}
