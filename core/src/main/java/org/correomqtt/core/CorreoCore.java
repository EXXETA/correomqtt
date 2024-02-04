package org.correomqtt.core;

import org.correomqtt.di.DefaultBean;
import org.correomqtt.core.eventbus.EventBus;
import org.correomqtt.core.settings.SettingsManager;

import org.correomqtt.di.Inject;

@DefaultBean
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
