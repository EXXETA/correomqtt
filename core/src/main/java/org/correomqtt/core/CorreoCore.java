package org.correomqtt.core;

import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.SoyEvents;
import org.correomqtt.core.settings.SettingsManager;

import org.correomqtt.di.Inject;

@DefaultBean
public class CorreoCore {

    private final SettingsManager settingsManager;
    private final SoyEvents soyEvents;

    @Inject
    public CorreoCore(SettingsManager settingsManager,
                      SoyEvents soyEvents) {
        this.settingsManager = settingsManager;
        this.soyEvents = soyEvents;
    }

    public void init() {
        System.setProperty("correo.configDirectory", settingsManager.getTargetDirectoryPath());
    }
}
