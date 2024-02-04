package org.correomqtt.core.shortcut;

import org.correomqtt.core.eventbus.Event;

public record ShortcutEvent(Shortcut shorcut) implements Event {

    public enum Shortcut {
        PUBLISH, SUBSCRIPTION, CLEAR_INCOMING, CLEAR_OUTGOING
    }
}
