package org.correomqtt.gui.contextmenu;

import org.correomqtt.business.services.ConfigService;
import org.correomqtt.gui.model.SubscriptionPropertiesDTO;
import javafx.event.ActionEvent;
import javafx.scene.control.MenuItem;
import javafx.scene.control.SeparatorMenuItem;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.ResourceBundle;

public class SubscriptionListMessageContextMenu extends BaseObjectContextMenu<SubscriptionPropertiesDTO, SubscriptionListMessageContextMenuDelegate> {

    private static final Logger LOGGER = LoggerFactory.getLogger(SubscriptionListMessageContextMenu.class);

    private MenuItem unsubscribe;
    private MenuItem filter;
    private MenuItem filterOnly;
    private MenuItem selectAll;
    private MenuItem selectNone;
    private MenuItem unsubscribeAll;

    private SeparatorMenuItem separator1;
    private SeparatorMenuItem separator2;
    private ResourceBundle resources;

    public SubscriptionListMessageContextMenu(SubscriptionListMessageContextMenuDelegate dispatcher) {
        super(dispatcher);
    }

    @Override
    protected void initializeItems() {
        resources = ResourceBundle.getBundle("org.correomqtt.i18n", ConfigService.getInstance().getSettings().getCurrentLocale());

        super.initializeItems();

        unsubscribe = new MenuItem("Unsubscribe");
        unsubscribe.setOnAction(this::unsubcribe);

        filter = new MenuItem("Toggle Filter");
        filter.setOnAction(this::toggleFilter);

        filterOnly = new MenuItem(resources.getString("subscriptionListMessageContextMenuFilterOnlyMenuItem"));
        filterOnly.setOnAction(this::filterOnly);

        selectAll = new MenuItem(resources.getString("subscriptionListMessageContextMenuSelectAllMenuItem"));
        selectAll.setOnAction(this::selectAll);

        selectNone = new MenuItem(resources.getString("subscriptionListMessageContextMenuSelectNoneMenuItem"));
        selectNone.setOnAction(this::selectNone);

        unsubscribeAll = new MenuItem(resources.getString("subscriptionListMessageContextMenuUnsubscribeAllMenuItem"));
        unsubscribeAll.setOnAction(this::unsubscribeAll);

        separator1 = new SeparatorMenuItem();
        separator2 = new SeparatorMenuItem();

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
