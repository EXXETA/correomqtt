package org.correomqtt.gui.controller;

import org.correomqtt.gui.model.ConnectionPropertiesDTO;

public interface ConnectionOnboardingDelegate {
    void onConnect(ConnectionPropertiesDTO config);
}
