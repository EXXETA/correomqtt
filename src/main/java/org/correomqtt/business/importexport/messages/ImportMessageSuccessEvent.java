package org.correomqtt.business.importexport.messages;

import org.correomqtt.business.eventbus.Event;
import org.correomqtt.business.model.MessageDTO;

public record ImportMessageSuccessEvent(MessageDTO messageDTO) implements Event {
}
