package org.correomqtt.business.shortcut;

import org.correomqtt.business.eventbus.Event;
import org.correomqtt.business.eventbus.SubscribeFilter;
import org.correomqtt.business.eventbus.SubscribeFilterNames;

public record ShortcutConnectionIdEvent(ShortcutEvent.Shortcut shortcut, String connectionId) implements Event {

    @SubscribeFilter(SubscribeFilterNames.CONNECTION_ID)
    public String getConnectionId(){
        return this.connectionId;
    }
}
