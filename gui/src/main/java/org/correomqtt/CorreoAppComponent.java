package org.correomqtt;

import dagger.BindsInstance;
import dagger.Component;
import javafx.application.HostServices;
import org.correomqtt.gui.model.AppHostServices;

import javax.inject.Singleton;

@Singleton
@Component()
public interface CorreoAppComponent {

    CorreoApp app();

    @Component.Builder
    interface Builder {
        @BindsInstance
        Builder hostServices(@AppHostServices HostServices hostServices);

        CorreoAppComponent build();
    }
}