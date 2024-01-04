package org.correomqtt.business.importexport.messages;

import org.correomqtt.business.eventbus.Event;

import java.io.File;

public record ImportMessageStartedEvent(File file) implements Event {
}
