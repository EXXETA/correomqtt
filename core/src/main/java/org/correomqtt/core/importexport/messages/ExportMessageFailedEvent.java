package org.correomqtt.core.importexport.messages;

import org.correomqtt.core.eventbus.Event;
import org.correomqtt.core.model.MessageDTO;

import java.io.File;

public record ExportMessageFailedEvent(File file, MessageDTO messageDTO, Throwable throwable) implements Event {
}
