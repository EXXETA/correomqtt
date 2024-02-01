package org.correomqtt;

import javafx.application.Application;
import javafx.geometry.Rectangle2D;
import javafx.stage.Screen;
import javafx.stage.Stage;
import lombok.extern.slf4j.Slf4j;
import org.correomqtt.core.utils.VersionUtils;

@Slf4j
public class FxApplication extends Application {
    private MainApplication app;

    @Override
    public void init() {

        startLog();

        MainComponent component = DaggerMainComponent.builder()
                .hostServices(getHostServices())
                .build();

        GuiCore.setMainComponent(component);

        app = component.mainApplication();
        app.onNotifyPreloader(this::notifyPreloader);
        app.init();
    }

    @Override
    public void start(Stage primaryStage) throws Exception {
        app.start(primaryStage);
    }

    private static void startLog() {

        if (log.isInfoEnabled()) {
            log.info("CorreoMQTT version is {}", VersionUtils.getVersion());
            log.info("JVM: {} {} {}", System.getProperty("java.vendor"), System.getProperty("java.runtime.name"), System.getProperty("java.runtime.version"));
            log.info("JavaFX: {}, Runtime: {}", System.getProperty("javafx.version"), System.getProperty("javafx.runtime.version"));
            log.info("OS: {} {} {}", System.getProperty("os.name"), System.getProperty("os.version"), System.getProperty("os.arch"));
            Rectangle2D screenBounds = Screen.getPrimary().getBounds();
            String xdgCurrentDesktop = System.getenv("XDG_CURRENT_DESKTOP");
            log.info("ENV: {}{} x {} ", xdgCurrentDesktop != null ? xdgCurrentDesktop + " " : "", screenBounds.getWidth(), screenBounds.getHeight());
        }
    }
}
