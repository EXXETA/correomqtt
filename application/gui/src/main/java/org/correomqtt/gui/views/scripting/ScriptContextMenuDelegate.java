package org.correomqtt.gui.views.scripting;

import org.correomqtt.gui.contextmenu.BaseObjectContextMenuDelegate;

public interface ScriptContextMenuDelegate extends BaseObjectContextMenuDelegate {
    void renameScript(ScriptFilePropertiesDTO dto);

    void deleteScript(ScriptFilePropertiesDTO dto);

    void runScript(ScriptFilePropertiesDTO dto);
}
