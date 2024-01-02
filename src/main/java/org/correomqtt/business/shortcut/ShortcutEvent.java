package org.correomqtt.business.shortcut;

import org.correomqtt.business.eventbus.Event;

public record ShortcutEvent(Shortcut shorcut) implements Event {

    public enum Shortcut {
        PUBLISH, SUBSCRIPTION, CLEAR_INCOMING, CLEAR_OUTGOING
    }
}
