package org.correomqtt.gui.views.connectionsettings;

import org.correomqtt.gui.model.ConnectionPropertiesDTO;

public interface ConnectionSettingsDelegateController {
    boolean saveConnection();

    void cleanUp();

    ConnectionPropertiesDTO getDTO();

    void setDTO(ConnectionPropertiesDTO config);

    void resetDTO();
}
