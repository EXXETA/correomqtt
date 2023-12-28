package org.correomqtt.gui.controller;

import org.correomqtt.gui.model.ConnectionPropertiesDTO;

public interface ConnectionSettingsDelegateController {
    boolean saveConnection();

    void cleanUp();

    ConnectionPropertiesDTO getDTO();

    void setDTO(ConnectionPropertiesDTO config);

    void resetDTO();
}
