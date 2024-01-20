package org.correomqtt.core.importexport.messages;

import org.correomqtt.core.eventbus.Event;

import java.io.File;

public record ImportMessageFailedEvent(File file, Throwable throwable) implements Event {
}
