package org.correomqtt.core;

import org.correomqtt.core.eventbus.EventBus;
import org.correomqtt.core.settings.SettingsManager;

import javax.inject.Inject;

public class CorreoCore {

    private final SettingsManager settingsManager;
    private final EventBus eventBus;

    @Inject
    public CorreoCore(SettingsManager settingsManager,
                      EventBus eventBus) {
        this.settingsManager = settingsManager;
        this.eventBus = eventBus;
    }

    public void init() {
        System.setProperty("correo.configDirectory", settingsManager.getTargetDirectoryPath());
    }
}
