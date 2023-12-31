package org.correomqtt.business.fileprovider;

import org.correomqtt.business.eventbus.Event;

public record SecretStoreErrorEvent(
        org.correomqtt.business.fileprovider.SecretStoreErrorEvent.Error error) implements Event {

    public enum Error {
        PASSWORD_FILE_UNREADABLE
    }
}
