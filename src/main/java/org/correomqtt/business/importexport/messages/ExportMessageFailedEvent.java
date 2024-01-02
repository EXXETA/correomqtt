package org.correomqtt.business.importexport.messages;

import org.correomqtt.business.eventbus.Event;
import org.correomqtt.business.model.MessageDTO;

import java.io.File;

public record ExportMessageFailedEvent(File file, MessageDTO messageDTO, Throwable throwable) implements Event {
}
