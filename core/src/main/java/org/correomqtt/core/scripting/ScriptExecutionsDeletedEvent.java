package org.correomqtt.core.scripting;

import lombok.AllArgsConstructor;
import lombok.Getter;
import org.correomqtt.di.Event;

@AllArgsConstructor
@Getter
public class ScriptExecutionsDeletedEvent implements Event {

    private String filename;
}
