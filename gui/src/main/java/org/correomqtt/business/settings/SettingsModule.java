package org.correomqtt.business.settings;

import dagger.Module;
import dagger.Provides;
import org.correomqtt.core.settings.CoreSettings;

@Module
public class SettingsModule {

    @Provides
    public CoreSettings createCoreSettings() {
        return SettingsProvider.getInstance(); //TODO
    }
}
