package org.correomqtt.gui.helper;

import org.correomqtt.business.services.ConfigService;
import javafx.application.Platform;
import javafx.scene.control.Alert;
import javafx.scene.control.Alert.AlertType;
import javafx.scene.control.ButtonType;
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

    private static boolean showConfirmationDialog(String title, String header, String content, String noButton, String yesButton) {
        Alert alert = new Alert(Alert.AlertType.CONFIRMATION);
        DialogPane dialogPane = alert.getDialogPane();
        String cssPath = ConfigService.getInstance().getCssPath();
        if (cssPath != null) {
            dialogPane.getStylesheets().add(cssPath);
        }

        alert.setTitle(title);
        alert.setHeaderText(header);
        alert.setContentText(content);
        ButtonType no = new ButtonType(noButton);
        ButtonType yes = new ButtonType(yesButton);

        alert.getButtonTypes().setAll(yes, no);

        return alert.showAndWait().get().equals(yes);
    }

    public static void info(String title, String content) {
        showDialog(INFORMATION, title, content);
    }

    public static void warn(String title, String content) {
        showDialog(WARNING, title, content);
    }

    public static boolean confirm(String title, String header, String content, String noButton, String yesButton) {
        return showConfirmationDialog(title, header, content, noButton, yesButton);
    }
}
