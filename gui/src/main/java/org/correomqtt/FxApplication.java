package org.correomqtt;

import javafx.application.Application;
import javafx.stage.Stage;
import lombok.Getter;

public class FxApplication extends Application {
    private CorreoApp app;

    @Getter
    private static CorreoAppComponent appComponent;

    @Override
    public void init() {
        appComponent = DaggerCorreoAppComponent.builder()
                .hostServices(getHostServices())
                .build();

        app = appComponent.app();
        app.onNotifyPreloader(this::notifyPreloader);
        app.init();
    }

    @Override
    public void start(Stage primaryStage) throws Exception {
        app.start(primaryStage);
    }
}
