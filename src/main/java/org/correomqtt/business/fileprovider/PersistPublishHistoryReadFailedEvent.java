package org.correomqtt.business.fileprovider;

import lombok.AllArgsConstructor;
import lombok.Getter;
import org.correomqtt.business.eventbus.Event;

@AllArgsConstructor
@Getter
public class PersistPublishHistoryReadFailedEvent implements Event {

    private Throwable exception;
}
