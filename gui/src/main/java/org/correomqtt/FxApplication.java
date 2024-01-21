package org.correomqtt;

import javafx.application.Application;
import javafx.stage.Stage;

public class FxApplication extends Application {
    private CorreoApp app;

    @Override
    public void init() {
        app = DaggerCorreoAppComponent.builder()
                .hostServices(getHostServices())
                .build().app();
        app.onNotifyPreloader(this::notifyPreloader);
        app.init();
    }

    @Override
    public void start(Stage primaryStage) throws Exception {
        app.start(primaryStage);
    }
}
