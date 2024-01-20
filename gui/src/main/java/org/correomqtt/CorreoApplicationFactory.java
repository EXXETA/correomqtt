package org.correomqtt;

import dagger.Component;
import org.correomqtt.business.settings.SettingsModule;

import javax.inject.Singleton;

@Singleton
@Component(modules = SettingsModule.class)
public interface CorreoApplicationFactory {
    CorreoApplication application();
}