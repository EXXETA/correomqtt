package org.correomqtt.core.importexport.messages;

import org.correomqtt.di.Event;

import java.io.File;

public record ImportMessageStartedEvent(File file) implements Event {
}
