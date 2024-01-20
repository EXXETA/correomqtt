package org.correomqtt;

import javafx.application.Application;
import javafx.stage.Stage;

public class JavaFXApplication extends Application {

    CorreoApplication app;

    public JavaFXApplication() {
        super();
      /*  app = DaggerCorreoApplicationFactory.builder()
                .settingsModule(new SettingsModule())
                .build();*/ //TODO
    }

    @Override
    public void init() {
        app.init(getHostServices());
    }

    @Override
    public void start(Stage primaryStage) throws Exception {
        app.start(primaryStage);
    }
}
