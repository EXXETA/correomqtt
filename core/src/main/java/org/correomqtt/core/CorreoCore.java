package org.correomqtt.core;

import org.correomqtt.core.settings.SettingsProvider;

import javax.inject.Inject;

public class CorreoCore {

    private final SettingsProvider settingsProvider;

    @Inject
    public CorreoCore(SettingsProvider settingsProvider) {
        this.settingsProvider = settingsProvider;
    }

    public void init() {
        System.setProperty("correo.configDirectory",settingsProvider.getTargetDirectoryPath());
    }
}
