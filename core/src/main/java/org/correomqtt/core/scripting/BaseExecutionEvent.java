package org.correomqtt.core.scripting;

import lombok.AllArgsConstructor;
import lombok.Getter;
import org.correomqtt.core.eventbus.Event;
import org.correomqtt.core.eventbus.SubscribeFilter;

import static org.correomqtt.core.eventbus.SubscribeFilterNames.SCRIPT_EXECUTION_ID;
import static org.correomqtt.core.eventbus.SubscribeFilterNames.SCRIPT_NAME;

@AllArgsConstructor
@Getter
public abstract class BaseExecutionEvent implements Event {

    private ExecutionDTO executionDTO;

    @SuppressWarnings("unused")
    @SubscribeFilter(SCRIPT_NAME)
    public String getFileName() {
        return executionDTO.getScriptFile().getName();
    }

    @SuppressWarnings("unused")
    @SubscribeFilter(SCRIPT_EXECUTION_ID)
    public String getExecutionId() {
        return executionDTO.getExecutionId();
    }
}
