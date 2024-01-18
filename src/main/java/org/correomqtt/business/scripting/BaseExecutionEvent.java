package org.correomqtt.business.scripting;

import lombok.AllArgsConstructor;
import lombok.Getter;
import org.correomqtt.business.eventbus.Event;
import org.correomqtt.business.eventbus.SubscribeFilter;

import static org.correomqtt.business.eventbus.SubscribeFilterNames.SCRIPT_EXECUTION_ID;
import static org.correomqtt.business.eventbus.SubscribeFilterNames.SCRIPT_NAME;

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
