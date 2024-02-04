package org.correomqtt.core.shortcut;

import org.correomqtt.di.Event;

public record ShortcutEvent(Shortcut shorcut) implements Event {

    public enum Shortcut {
        PUBLISH, SUBSCRIPTION, CLEAR_INCOMING, CLEAR_OUTGOING
    }
}
