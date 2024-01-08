package org.correomqtt.gui.views.scripting;

import org.correomqtt.gui.model.ConnectionPropertiesDTO;

public interface SingleEditorViewDelegate {
    void renameScript(ScriptFilePropertiesDTO scriptFilePropertiesDTO);

    void deleteScript(ScriptFilePropertiesDTO scriptFilePropertiesDTO);

    boolean addExecution(ScriptFilePropertiesDTO dto, ConnectionPropertiesDTO selectedConnection, String scriptCode);
}