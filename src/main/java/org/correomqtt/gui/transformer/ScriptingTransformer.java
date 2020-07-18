package org.correomqtt.gui.transformer;

import org.correomqtt.business.model.ScriptingDTO;
import org.correomqtt.gui.model.ScriptingPropertiesDTO;

public class ScriptingTransformer {

    private ScriptingTransformer() {
        //private constructor
    }

    public static ScriptingPropertiesDTO dtoToProps(ScriptingDTO scriptingDTO) {
        return ScriptingPropertiesDTO.builder()
                                   .name(scriptingDTO.getName())
                                   .path(scriptingDTO.getPath())
                                   .build();
    }

    public static ScriptingDTO propsToDTO(ScriptingPropertiesDTO scriptingPropertiesDTO) {
        return ScriptingDTO.builder()
                .name(scriptingPropertiesDTO.getName())
                .path(scriptingPropertiesDTO.getPath())
                .build();
    }
}
