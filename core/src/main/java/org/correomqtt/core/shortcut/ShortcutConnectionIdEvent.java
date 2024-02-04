package org.correomqtt.core.shortcut;

import org.correomqtt.core.eventbus.Event;
import org.correomqtt.core.eventbus.SubscribeFilter;
import org.correomqtt.core.eventbus.SubscribeFilterNames;

public record ShortcutConnectionIdEvent(ShortcutEvent.Shortcut shortcut, String connectionId) implements Event {

    @SubscribeFilter(SubscribeFilterNames.CONNECTION_ID)
    public String getConnectionId(){
        return this.connectionId;
    }
}
