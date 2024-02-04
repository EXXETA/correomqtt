package org.correomqtt.core.importexport.messages;

import org.correomqtt.core.eventbus.Event;

import java.io.File;

public record ImportMessageStartedEvent(File file) implements Event {
}
