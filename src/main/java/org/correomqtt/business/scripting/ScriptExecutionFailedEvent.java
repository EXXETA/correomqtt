package org.correomqtt.business.scripting;

import org.correomqtt.business.eventbus.Event;
import org.correomqtt.business.eventbus.SubscribeFilter;

import static org.correomqtt.business.eventbus.SubscribeFilterNames.SCRIPT_EXECUTION_ID;

public class ScriptExecutionFailedEvent extends BaseExecutionEvent {

    public ScriptExecutionFailedEvent(ExecutionDTO executionDTO) {
        super(executionDTO);
    }
}
