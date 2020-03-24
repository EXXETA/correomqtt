package org.correomqtt;

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
import org.correomqtt.business.services.ConfigService;
import org.correomqtt.business.utils.VersionUtils;
import org.correomqtt.gui.controller.PreloaderViewController;
import java.io.IOException;
import java.util.ResourceBundle;

public class CorreoPreloader extends Preloader implements PreloadingObserver {

    public CorreoPreloader() {
        PreloadingDispatcher.getInstance().addObserver(this);
    }

    PreloaderViewController preloaderViewController;
    private Scene scene;
    private Stage preloaderStage;
    private ResourceBundle resources;

    @Override
    public void init() throws IOException {
        ConfigService.getInstance().setCssFileName();
        String cssPath = ConfigService.getInstance().getCssPath();

        FXMLLoader loader = new FXMLLoader(PreloaderViewController.class.getResource("preloaderView.fxml"));
        Parent root = loader.load();

        preloaderViewController = loader.getController();
        preloaderViewController.getPreloaderVersionLabel().setText("v" + VersionUtils.getVersion());
        scene = new Scene(root, 500, 300);

        if (cssPath != null) {
            scene.getStylesheets().add(cssPath);
        }
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
