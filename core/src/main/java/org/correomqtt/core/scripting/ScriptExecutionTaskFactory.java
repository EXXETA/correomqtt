package org.correomqtt.core.scripting;

import dagger.assisted.AssistedFactory;

@AssistedFactory
public interface ScriptExecutionTaskFactory {
    ScriptExecutionTask create(ExecutionDTO executionDTO);
}