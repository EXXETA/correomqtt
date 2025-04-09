package org.correomqtt.core.shortcut;

import org.correomqtt.di.Event;
import org.correomqtt.di.ObservesFilter;
import org.correomqtt.core.events.ObservesFilterNames;

public record ShortcutConnectionIdEvent(ShortcutEvent.Shortcut shortcut, String connectionId) implements Event {

    @ObservesFilter(ObservesFilterNames.CONNECTION_ID)
    public String getConnectionId(){
        return this.connectionId;
    }
}
