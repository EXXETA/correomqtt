package org.correomqtt;

import javafx.application.Preloader;
import javafx.fxml.FXMLLoader;
import javafx.scene.Parent;
import javafx.scene.Scene;
import javafx.stage.Stage;
import javafx.stage.StageStyle;
import org.correomqtt.business.services.ConfigService;
import org.correomqtt.business.utils.VersionUtils;
import org.correomqtt.gui.controller.PreloaderViewController;
import java.io.IOException;
import java.util.ResourceBundle;

public class CorreoPreloader extends Preloader {

    PreloaderViewController preloaderViewController;
    private Scene scene;
    private Stage preloaderStage;
    private ResourceBundle resources;

    @Override
    public void init() throws IOException {
        FXMLLoader loader = new FXMLLoader(PreloaderViewController.class.getResource("preloaderView.fxml"));
        Parent root = loader.load();

        preloaderViewController = loader.getController();
        preloaderViewController.getVersionLabel().setText("v" + VersionUtils.getVersion());
        scene = new Scene(root, 500, 300);
    }

    @Override
    public void start(Stage primaryStage) throws Exception {
        this.preloaderStage = primaryStage;
        preloaderStage.setScene(scene);
        preloaderStage.initStyle(StageStyle.UNDECORATED);
        preloaderStage.show();
    }

    @Override
    public void handleApplicationNotification(PreloaderNotification info) {
        if (info instanceof ProgressNotification) {
            String text = "";
            switch ((int) ((ProgressNotification) info).getProgress()) {
                case 0:
                    resources = ResourceBundle.getBundle("org.correomqtt.i18n", ConfigService.getInstance().getSettings().getCurrentLocale());
                    text = resources.getString("preloaderLanguageSet");
                    break;
                case 10: text = resources.getString("preloaderSearchingUpdates"); break;
                case 20: text = resources.getString("preloaderReady"); break;
            }
            preloaderViewController.getPreloaderStepLabel().setText(text);
        }
    }

    @Override
    public void handleStateChangeNotification(StateChangeNotification stateChangeNotification) {
        if (stateChangeNotification.getType() == StateChangeNotification.Type.BEFORE_START) {
            preloaderStage.hide();
        }
    }
}
