package org.correomqtt.gui.contextmenu;

import org.correomqtt.di.Assisted;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import javafx.scene.control.SeparatorMenuItem;
import org.correomqtt.core.settings.SettingsManager;

@SuppressWarnings("java:S110")
@DefaultBean
public class DetailContextMenu extends BaseMessageContextMenu<DetailContextMenuDelegate> {

    private SeparatorMenuItem separator;

    @Inject
    public DetailContextMenu(SettingsManager settingsManager,
                             @Assisted DetailContextMenuDelegate dispatcher) {
        super(settingsManager, dispatcher);
    }

    @Override
    protected void initializeItems() {
        super.initializeItems();
        separator = new SeparatorMenuItem();
        this.getItems().addAll(putToForm,
                showDetails,
                separator,
                copyTopicToClipboard,
                copyTimeToClipboard,
                copyPayloadToClipboard);
        delegate.isInlineView().addListener((observable, oldValue, newValue) -> updateInlineView());
        updateInlineView();
    }

    private void updateInlineView() {
        showDetails.setVisible(delegate.isInlineView().getValue());
    }

    @Override
    protected void setVisibilityForObjectItems(boolean visible) {
        super.setVisibilityForObjectItems(visible);
        updateInlineView();
        separator.setVisible(visible);
    }
}
