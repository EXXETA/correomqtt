package org.correomqtt.gui.transformer;

import org.correomqtt.business.scripting.ExecutionDTO;
import org.correomqtt.gui.model.ExecutionPropertiesDTO;

public class ExecutionTransformer {

    private ExecutionTransformer() {
        //private constructor
    }

    public static ExecutionPropertiesDTO dtoToProps(ExecutionDTO executionDTO) {
        return ExecutionPropertiesDTO.builder()
                                   .executionId(executionDTO.getScriptExecutionDTO().getExecutionId())
                                   .build();
    }
}
