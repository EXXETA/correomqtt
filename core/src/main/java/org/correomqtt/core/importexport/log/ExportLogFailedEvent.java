package org.correomqtt.core.importexport.log;

import org.correomqtt.di.Event;

import java.io.File;

public record ExportLogFailedEvent(File file, String logs, Throwable throwable) implements Event {
}
