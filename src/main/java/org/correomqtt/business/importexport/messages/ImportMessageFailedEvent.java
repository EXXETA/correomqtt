package org.correomqtt.business.importexport.messages;

import org.correomqtt.business.eventbus.Event;

import java.io.File;

public record ImportMessageFailedEvent(File file, Throwable throwable) implements Event {
}
