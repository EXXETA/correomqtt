package org.correomqtt.core.fileprovider;

import lombok.AllArgsConstructor;
import lombok.Getter;
import org.correomqtt.di.Event;

@AllArgsConstructor
@Getter
public class PersistPublishHistoryWriteFailedEvent implements Event {
    private Throwable exception;
}
