package org.correomqtt.core.importexport.log;

import org.correomqtt.di.Event;

import java.io.File;

public record ExportLogStartedEvent(File file, String logs) implements Event {
}
