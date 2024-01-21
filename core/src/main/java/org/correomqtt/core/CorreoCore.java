package org.correomqtt.core;

import org.correomqtt.core.settings.SettingsManager;

import javax.inject.Inject;

public class CorreoCore {

    private final SettingsManager settingsManager;

    @Inject
    public CorreoCore(SettingsManager settingsManager) {
        this.settingsManager = settingsManager;
    }

    public void init() {
        System.setProperty("correo.configDirectory", settingsManager.getTargetDirectoryPath());
    }
}
