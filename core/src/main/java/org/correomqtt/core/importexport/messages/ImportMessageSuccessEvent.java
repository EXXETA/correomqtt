package org.correomqtt.core.importexport.messages;

import org.correomqtt.core.eventbus.Event;
import org.correomqtt.core.model.MessageDTO;

public record ImportMessageSuccessEvent(MessageDTO messageDTO) implements Event {
}
