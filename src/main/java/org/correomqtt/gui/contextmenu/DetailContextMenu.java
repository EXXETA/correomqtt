package org.correomqtt.gui.contextmenu;

import javafx.scene.control.SeparatorMenuItem;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class DetailContextMenu extends BaseMessageContextMenu<DetailContextMenuDelegate> {

    private static final Logger LOGGER = LoggerFactory.getLogger(DetailContextMenu.class);

    private SeparatorMenuItem separator;

    public DetailContextMenu(DetailContextMenuDelegate dispatcher) {
        super(dispatcher);
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
