package com.exxeta.correomqtt.gui.controller;

import com.exxeta.correomqtt.gui.model.ConnectionPropertiesDTO;

public interface ConnectionOnboardingDelegate {
    void onConnect(ConnectionPropertiesDTO config);
}
