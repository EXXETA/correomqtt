package org.correomqtt.gui.views.onboarding;

import org.correomqtt.gui.model.ConnectionPropertiesDTO;

public interface ConnectionOnboardingDelegate {
    void onConnect(ConnectionPropertiesDTO config);

    void cleanUpProvider(ConnectionPropertiesDTO config);
}
