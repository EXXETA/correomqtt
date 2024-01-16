package org.correomqtt.business.scripting;

import lombok.AllArgsConstructor;
import lombok.Getter;
import org.correomqtt.business.eventbus.Event;

@AllArgsConstructor
@Getter
public class ScriptExecutionsDeletedEvent implements Event {

    private String filename;
}
