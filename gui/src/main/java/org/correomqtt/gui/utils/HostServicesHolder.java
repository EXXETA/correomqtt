package org.correomqtt.gui.utils;

import javafx.application.HostServices;

public class HostServicesHolder {

    private static HostServicesHolder instance;
    private HostServices hostServices;

    private HostServicesHolder() {
        // empty constructor
    }

    public static synchronized HostServicesHolder getInstance() {
        if (instance == null) {
            instance = new HostServicesHolder();
        }
        return instance;
    }

    public HostServices getHostServices() {
        return hostServices;
    }

    public void setHostServices(HostServices hostServices) {
        this.hostServices = hostServices;
    }
}
