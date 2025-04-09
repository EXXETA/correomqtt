package org.correomqtt.gui.views.scripting;

import org.correomqtt.gui.model.ConnectionPropertiesDTO;

public interface SingleEditorViewDelegate {
    void renameScript(ScriptFilePropertiesDTO dto);

    void deleteScript(ScriptFilePropertiesDTO dto);

    boolean addExecution(ScriptFilePropertiesDTO dto, ConnectionPropertiesDTO selectedConnection, String scriptCode);

    void onPlainTextChange(ScriptFilePropertiesDTO dto);
}