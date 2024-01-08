package org.correomqtt.gui.views.scripting;

import org.correomqtt.business.scripting.ExecutionDTO;

public class ExecutionTransformer {

    private ExecutionTransformer() {
        //private constructor
    }

    public static ExecutionPropertiesDTO dtoToProps(ExecutionDTO executionDTO) {
        return ExecutionPropertiesDTO.builder()
                .executionId(executionDTO.getExecutionId())
                .connectionId(executionDTO.getConnectionId())
                .scriptFile(ScriptingTransformer.dtoToProps(executionDTO.getScriptFile()))
                .jsCode(executionDTO.getJsCode())
                .error(executionDTO.getError())
                .startTime(executionDTO.getStartTime())
                .executionTime(executionDTO.getExecutionTime())
                .cancelled(executionDTO.isCancelled())
                .build();
    }

    public static void updatePropsByDto(ExecutionPropertiesDTO props, ExecutionDTO dto) {
        props.getErrorProperty().setValue(dto.getError());
        props.getStartTimeProperty().setValue(dto.getStartTime());
        props.getExecutionTimeProperty().setValue(dto.getExecutionTime());
        props.getCancelledProperty().setValue(dto.isCancelled());
    }
}
