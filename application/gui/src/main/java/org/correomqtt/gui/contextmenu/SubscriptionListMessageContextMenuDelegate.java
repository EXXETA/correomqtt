package org.correomqtt.gui.contextmenu;

import org.correomqtt.gui.model.SubscriptionPropertiesDTO;

public interface SubscriptionListMessageContextMenuDelegate extends BaseObjectContextMenuDelegate {
    void unsubscribeAll();

    void selectNone();

    void selectAll();

    void filterOnly(SubscriptionPropertiesDTO dto);

    void unsubscribe(SubscriptionPropertiesDTO dto);
}
