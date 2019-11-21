package com.exxeta.correomqtt.gui.helper;

import com.exxeta.correomqtt.business.services.ConfigService;
import javafx.application.Platform;
import javafx.scene.control.Alert;
import javafx.scene.control.Alert.AlertType;
import javafx.scene.control.DialogPane;

import static javafx.scene.control.Alert.AlertType.INFORMATION;
import static javafx.scene.control.Alert.AlertType.WARNING;

public class AlertHelper {

    private AlertHelper() {
        // Nothing to do here
    }

    private static void showDialog(AlertType type, String title, String content) {
        Platform.runLater(() -> {
            Alert alert = new Alert(type);
            DialogPane dialogPane = alert.getDialogPane();
            String cssPath = ConfigService.getInstance().getCssPath(false);
            if (cssPath != null) {
                dialogPane.getStylesheets().add(cssPath);
            }
            alert.setTitle(title);
            alert.setHeaderText(null);
            alert.setContentText(content);

            alert.showAndWait();
        });
    }

    public static void info(String title, String content) {
        showDialog(INFORMATION, title, content);
    }

    public static void warn(String title, String content) {
        showDialog(WARNING, title, content);
    }
}
