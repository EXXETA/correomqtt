package org.correomqtt.core.scripting;

import lombok.AllArgsConstructor;
import lombok.Getter;
import org.correomqtt.di.Event;
import org.correomqtt.di.ObservesFilter;

import static org.correomqtt.core.events.ObservesFilterNames.SCRIPT_EXECUTION_ID;
import static org.correomqtt.core.events.ObservesFilterNames.SCRIPT_NAME;

@AllArgsConstructor
@Getter
public abstract class BaseExecutionEvent implements Event {

    private ExecutionDTO executionDTO;

    @SuppressWarnings("unused")
    @ObservesFilter(SCRIPT_NAME)
    public String getFileName() {
        return executionDTO.getScriptFile().getName();
    }

    @SuppressWarnings("unused")
    @ObservesFilter(SCRIPT_EXECUTION_ID)
    public String getExecutionId() {
        return executionDTO.getExecutionId();
    }
}
