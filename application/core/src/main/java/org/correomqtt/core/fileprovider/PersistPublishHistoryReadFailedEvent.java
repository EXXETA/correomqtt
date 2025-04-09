package org.correomqtt.core.fileprovider;

import lombok.AllArgsConstructor;
import lombok.Getter;
import org.correomqtt.di.Event;

@AllArgsConstructor
@Getter
public class PersistPublishHistoryReadFailedEvent implements Event {

    private Throwable exception;
}
