package org.correomqtt;

import ch.qos.logback.classic.LoggerContext;
import ch.qos.logback.classic.util.ContextInitializer;
import ch.qos.logback.core.joran.spi.JoranException;
import ch.qos.logback.core.util.StatusPrinter;
import javafx.animation.Animation;
import javafx.animation.Interpolator;
import javafx.animation.RotateTransition;
import javafx.application.Platform;
import javafx.application.Preloader;
import javafx.fxml.FXMLLoader;
import javafx.scene.Parent;
import javafx.scene.Scene;
import javafx.scene.transform.Rotate;
import javafx.stage.Stage;
import javafx.stage.StageStyle;
import javafx.util.Duration;
import org.correomqtt.business.dispatcher.PreloadingDispatcher;
import org.correomqtt.business.dispatcher.PreloadingObserver;
import org.correomqtt.business.services.SettingsService;
import org.correomqtt.business.utils.VersionUtils;
import org.correomqtt.gui.controller.PreloaderViewController;
import org.slf4j.LoggerFactory;

import java.io.IOException;

public class CorreoPreloader extends Preloader implements PreloadingObserver {

    PreloaderViewController preloaderViewController;
    private Scene scene;
    private Stage preloaderStage;

    public CorreoPreloader() {
        PreloadingDispatcher.getInstance().addObserver(this);
    }

    @Override
    public void init() throws IOException {
        setLoggerFilePath();

        String cssPath = SettingsService.getInstance().getCssPath();

        FXMLLoader loader = new FXMLLoader(PreloaderViewController.class.getResource("preloaderView.fxml"));
        Parent root = loader.load();

        preloaderViewController = loader.getController();
        preloaderViewController.getPreloaderVersionLabel().setText("v" + VersionUtils.getVersion());
        scene = new Scene(root, 500, 300);

        if (cssPath != null) {
            scene.getStylesheets().add(cssPath);
        }
    }

    private void setLoggerFilePath() {
        // Set the path for file logging to user directory.
        System.setProperty("correomqtt-logfile", SettingsService.getInstance().getLogPath());
        System.setProperty(ContextInitializer.CONFIG_FILE_PROPERTY, CorreoMqtt.class.getResource("logger-config.xml").getPath());

        LoggerContext lc = (LoggerContext) LoggerFactory.getILoggerFactory();
        ContextInitializer ci = new ContextInitializer(lc);
        lc.reset();
        try {
            //I prefer autoConfig() over JoranConfigurator.doConfigure() so I wouldn't need to find the file myself.
            ci.autoConfig();
        } catch (JoranException e) {
            // StatusPrinter will try to log this
            e.printStackTrace(); //TODO
        }
        StatusPrinter.printInCaseOfErrorsOrWarnings(lc);
    }

    @Override
    public void start(Stage primaryStage) throws Exception {
        this.preloaderStage = primaryStage;
        preloaderStage.setScene(scene);
        preloaderStage.initStyle(StageStyle.UNDECORATED);

        RotateTransition rotateTransition = new RotateTransition();
        rotateTransition.setAxis(Rotate.Z_AXIS);
        rotateTransition.setByAngle(360);
        rotateTransition.setCycleCount(Animation.INDEFINITE);
        rotateTransition.setDuration(Duration.millis(1000));
        rotateTransition.setNode(preloaderViewController.getPreloaderProgressLabel());
        rotateTransition.setInterpolator(Interpolator.LINEAR);
        rotateTransition.play();

        preloaderStage.show();
    }

    @Override
    public void handleStateChangeNotification(StateChangeNotification stateChangeNotification) {
        if (stateChangeNotification.getType() == StateChangeNotification.Type.BEFORE_START) {
            preloaderStage.hide();
        }
    }

    @Override
    public void onProgress(Double progress, String message) {
        Platform.runLater(() -> {
            preloaderViewController.getPreloaderStepLabel().setText(message);
        });
    }

}
