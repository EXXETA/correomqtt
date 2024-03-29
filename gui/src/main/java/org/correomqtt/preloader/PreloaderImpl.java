package org.correomqtt.preloader;

import javafx.animation.Animation;
import javafx.animation.Interpolator;
import javafx.animation.RotateTransition;
import javafx.application.Preloader;
import javafx.fxml.FXML;
import javafx.fxml.FXMLLoader;
import javafx.scene.Parent;
import javafx.scene.Scene;
import javafx.scene.control.Label;
import javafx.scene.transform.Rotate;
import javafx.stage.Stage;
import javafx.util.Duration;
import org.correomqtt.core.utils.VersionUtils;
import org.correomqtt.gui.window.StageHelper;

import java.io.IOException;

public class PreloaderImpl extends Preloader {

    //TODO use a nice image for splash

    private Scene scene;
    private Stage preloaderStage;
    @FXML
    private Label preloaderProgressLabel;
    @FXML
    private Label preloaderStepLabel;
    @FXML
    private Label preloaderVersionLabel;

    @Override
    public void init() throws IOException {
        FXMLLoader loader = new FXMLLoader(PreloaderImpl.class.getResource("preloaderView.fxml"));
        loader.setControllerFactory(param -> this);
        Parent root = loader.load();

        preloaderVersionLabel.setText("v" + VersionUtils.getVersion());
        scene = new Scene(root, 500, 300);
    }

    @Override
    public void start(Stage primaryStage) throws Exception {
        this.preloaderStage = primaryStage;
        preloaderStage.setScene(scene);

        StageHelper.enforceFloatingWindow(preloaderStage);

        RotateTransition rotateTransition = new RotateTransition();
        rotateTransition.setAxis(Rotate.Z_AXIS);
        rotateTransition.setByAngle(360);
        rotateTransition.setCycleCount(Animation.INDEFINITE);
        rotateTransition.setDuration(Duration.millis(1000));
        rotateTransition.setNode(preloaderProgressLabel);
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
    public void handleApplicationNotification(PreloaderNotification info) {
        if (info instanceof org.correomqtt.preloader.PreloaderNotification cInfo) {
            preloaderStepLabel.setText(cInfo.getMsg());
        }
    }
}
