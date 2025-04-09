package org.correomqtt.core.importexport.messages;

import org.correomqtt.di.Event;
import org.correomqtt.core.model.MessageDTO;

public record ImportMessageSuccessEvent(MessageDTO messageDTO) implements Event {
}
