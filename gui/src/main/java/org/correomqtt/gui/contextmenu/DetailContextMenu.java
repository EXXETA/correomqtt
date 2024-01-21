package org.correomqtt.gui.contextmenu;

import dagger.assisted.Assisted;
import dagger.assisted.AssistedFactory;
import dagger.assisted.AssistedInject;
import javafx.scene.control.SeparatorMenuItem;
import org.correomqtt.core.settings.SettingsProvider;

@SuppressWarnings("java:S110")
public class DetailContextMenu extends BaseMessageContextMenu<DetailContextMenuDelegate> {

    private SeparatorMenuItem separator;

    @AssistedFactory
    public interface Factory {
        DetailContextMenu create(DetailContextMenuDelegate delegate);

    }
    @AssistedInject
    public DetailContextMenu(
            SettingsProvider settingsProvider,
            @Assisted DetailContextMenuDelegate dispatcher) {
        super(settingsProvider, dispatcher);
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
