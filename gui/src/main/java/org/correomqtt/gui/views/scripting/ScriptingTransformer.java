package org.correomqtt.gui.views.scripting;

import org.correomqtt.core.scripting.ScriptFileDTO;

public class ScriptingTransformer {

    private ScriptingTransformer() {
        //private constructor
    }

    public static ScriptFilePropertiesDTO dtoToProps(ScriptFileDTO scriptFileDTO) {
        return  ScriptFilePropertiesDTO.builder()
                                   .name(scriptFileDTO.getName())
                                   .path(scriptFileDTO.getPath())
                                   .build();
    }

    public static ScriptFileDTO propsToDTO(ScriptFilePropertiesDTO scriptFilePropertiesDTO) {
        return ScriptFileDTO.builder()
                .name(scriptFilePropertiesDTO.getName())
                .path(scriptFilePropertiesDTO.getPath())
                .build();
    }
}
