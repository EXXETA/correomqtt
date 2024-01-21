package org.correomqtt.gui.contextmenu;

import dagger.assisted.Assisted;
import dagger.assisted.AssistedFactory;
import dagger.assisted.AssistedInject;
import javafx.event.ActionEvent;
import javafx.scene.control.MenuItem;
import javafx.scene.control.SeparatorMenuItem;
import org.correomqtt.core.settings.SettingsProvider;
import org.correomqtt.gui.model.SubscriptionPropertiesDTO;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.ResourceBundle;

@SuppressWarnings("java:S110")
public class SubscriptionListMessageContextMenu extends BaseObjectContextMenu<SubscriptionPropertiesDTO, SubscriptionListMessageContextMenuDelegate> {

    private static final Logger LOGGER = LoggerFactory.getLogger(SubscriptionListMessageContextMenu.class);

    private MenuItem unsubscribe;
    private MenuItem filter;
    private MenuItem filterOnly;

    private SeparatorMenuItem separator1;

    @AssistedFactory
    public interface Factory {
        SubscriptionListMessageContextMenu create(SubscriptionListMessageContextMenuDelegate delegate);

    }
    @AssistedInject
    public SubscriptionListMessageContextMenu(SettingsProvider settingsProvider,
                                              @Assisted SubscriptionListMessageContextMenuDelegate dispatcher) {
        super(settingsProvider, dispatcher);
    }

    @Override
    protected void initializeItems() {
        ResourceBundle resources = ResourceBundle.getBundle("org.correomqtt.i18n", settingsProvider.getSettings().getCurrentLocale());

        super.initializeItems();

        unsubscribe = new MenuItem("Unsubscribe");
        unsubscribe.setOnAction(this::unsubcribe);

        filter = new MenuItem("Toggle Filter");
        filter.setOnAction(this::toggleFilter);

        filterOnly = new MenuItem(resources.getString("subscriptionListMessageContextMenuFilterOnlyMenuItem"));
        filterOnly.setOnAction(this::filterOnly);

        MenuItem selectAll = new MenuItem(resources.getString("subscriptionListMessageContextMenuSelectAllMenuItem"));
        selectAll.setOnAction(this::selectAll);

        MenuItem selectNone = new MenuItem(resources.getString("subscriptionListMessageContextMenuSelectNoneMenuItem"));
        selectNone.setOnAction(this::selectNone);

        MenuItem unsubscribeAll = new MenuItem(resources.getString("subscriptionListMessageContextMenuUnsubscribeAllMenuItem"));
        unsubscribeAll.setOnAction(this::unsubscribeAll);

        separator1 = new SeparatorMenuItem();
        SeparatorMenuItem separator2 = new SeparatorMenuItem();

        this.getItems().addAll(unsubscribe,
                filter,
                filterOnly,
                separator1,
                selectAll,
                selectNone,
                separator2,
                unsubscribeAll);
    }

    @Override
    protected void setVisibilityForObjectItems(boolean visible) {
        super.setVisibilityForObjectItems(visible);
        unsubscribe.setVisible(visible);
        filter.setVisible(visible);
        filterOnly.setVisible(visible);
        separator1.setVisible(visible);
    }

    private void unsubscribeAll(ActionEvent actionEvent) {
        delegate.unsubscribeAll();

    }

    private void selectNone(ActionEvent actionEvent) {
        delegate.selectNone();
    }

    private void selectAll(ActionEvent actionEvent) {
        delegate.selectAll();
    }

    private void filterOnly(ActionEvent actionEvent) {
        if (dto != null) {
            delegate.filterOnly(this.dto);
        } else {
            LOGGER.warn("Call to {}::filterOnly with empty message.", getClassName());
        }
    }

    private void toggleFilter(ActionEvent actionEvent) {
        if (dto != null) {
            dto.setFiltered(!dto.isFiltered());
        } else {
            LOGGER.warn("Call to {}::toggleFilter with empty message.", getClassName());
        }
    }

    private void unsubcribe(ActionEvent actionEvent) {
        if (dto != null) {
            delegate.unsubscribe(dto);
        } else {
            LOGGER.warn("Call to {}::unsubcribe with empty message.", getClassName());
        }
    }
}
