package org.correomqtt.gui.helper;

import javafx.collections.FXCollections;
import javafx.geometry.Pos;
import javafx.scene.control.Alert;
import javafx.scene.control.Alert.AlertType;
import javafx.scene.control.ButtonType;
import javafx.scene.control.ComboBox;
import javafx.scene.control.Dialog;
import javafx.scene.control.DialogPane;
import javafx.scene.control.Label;
import javafx.scene.control.PasswordField;
import javafx.scene.layout.Region;
import javafx.scene.layout.VBox;
import javafx.stage.Stage;
import javafx.stage.StageStyle;
import javafx.stage.Window;
import org.correomqtt.business.provider.SettingsProvider;
import org.correomqtt.gui.utils.PlatformUtils;
import org.correomqtt.gui.window.StageHelper;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.List;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicReference;

import static javafx.scene.control.Alert.AlertType.INFORMATION;
import static javafx.scene.control.Alert.AlertType.WARNING;

public class AlertHelper {

    private static final Logger LOGGER = LoggerFactory.getLogger(AlertHelper.class);

    private AlertHelper() {
        // Nothing to do here
    }

    private static void showDialog(AlertType type, String title, String content, boolean block, ButtonType buttonType) {
        final CountDownLatch countDownLatch = block ? new CountDownLatch(1) : null;
        PlatformUtils.runLaterIfNotInFxThread(() -> {
            Alert alert = new Alert(type);
            StageHelper.enforceFloatingWindow(alert);
            DialogPane dialogPane = alert.getDialogPane();
            String cssPath = SettingsProvider.getInstance().getCssPath();
            if (cssPath != null) {
                dialogPane.getStylesheets().add(cssPath);
            }
            alert.setTitle(title);
            alert.initOwner(Window.getWindows().stream().filter(Window::isShowing).findFirst().orElse(null));
            alert.setHeaderText(null);
            alert.setContentText(content);
            alert.getDialogPane().setMinHeight(Region.USE_PREF_SIZE);
            if (buttonType != null) {
                alert.getButtonTypes().setAll(buttonType);
            }
            alert.initStyle(StageStyle.UTILITY);
            alert.showAndWait();
            if (countDownLatch != null) {
                countDownLatch.countDown();
            }
        });
        if (countDownLatch != null) {
            try {
                countDownLatch.await();
            } catch (InterruptedException e) {
                LOGGER.error("Exception during showing dialog ", e);
                Thread.currentThread().interrupt();
            }
        }
    }

    private static boolean showConfirmationDialog(String title, String header, String content, String noButton, String yesButton) {
        final CountDownLatch countDownLatch = new CountDownLatch(1);
        AtomicBoolean result = new AtomicBoolean();
        PlatformUtils.runLaterIfNotInFxThread(() -> {
            Alert alert = new Alert(Alert.AlertType.CONFIRMATION);
            StageHelper.enforceFloatingWindow(alert);
            DialogPane dialogPane = alert.getDialogPane();
            dialogPane.setMaxWidth(450);
            String cssPath = SettingsProvider.getInstance().getCssPath();
            if (cssPath != null) {
                dialogPane.getStylesheets().add(cssPath);
            }
            alert.setTitle(title);
            alert.setHeaderText(header);
            alert.setContentText(content);
            alert.getDialogPane().setMinHeight(Region.USE_PREF_SIZE);
            alert.initStyle(StageStyle.UTILITY);
            ButtonType no = new ButtonType(noButton);
            ButtonType yes = new ButtonType(yesButton);

            if (noButton != null) {
                alert.getButtonTypes().setAll(yes, no);
            } else {
                alert.getButtonTypes().setAll(yes);
            }

            alert.showAndWait().ifPresentOrElse(buttonType -> result.set(buttonType.equals(yes)), () -> result.set(false));

            countDownLatch.countDown();
        });
        try {
            countDownLatch.await();
        } catch (InterruptedException e) {
            LOGGER.error("Exception during confirm dialog ", e);
            Thread.currentThread().interrupt();
        }
        return result.get();

    }

    public static void info(String title, String content) {
        info(title, content, false);
    }

    public static void info(String title, String content, boolean block) {
        showDialog(INFORMATION, title, content, block, null);
    }

    public static void info(String title, String content, boolean block, ButtonType buttonType) {
        showDialog(INFORMATION, title, content, block, buttonType);
    }

    public static void warn(String title, String content) {
        warn(title, content, false);
    }

    public static void warn(String title, String content, boolean block) {
        showDialog(WARNING, title, content, block, null);
    }

    public static void warn(String title, String content, boolean block, ButtonType buttonType) {
        showDialog(WARNING, title, content, block, buttonType);
    }

    public static boolean confirm(String title, String header, String content, String noButton, String yesButton) {
        return showConfirmationDialog(title, header, content, noButton, yesButton);
    }

    public static String passwordInput(String title, String header, String content) {
        final CountDownLatch countDownLatch = new CountDownLatch(1);
        AtomicReference<String> result = new AtomicReference<>();
        PlatformUtils.runLaterIfNotInFxThread(() -> {
            Dialog<String> dialog = new Dialog<>();
            StageHelper.enforceFloatingWindow(dialog);
            dialog.setWidth(450);
            DialogPane dialogPane = dialog.getDialogPane();
            dialog.setWidth(450);
            dialogPane.setMinHeight(Region.USE_PREF_SIZE);
            String cssPath = SettingsProvider.getInstance().getCssPath();
            if (cssPath != null) {
                dialogPane.getStylesheets().add(cssPath);
            }
            dialog.setTitle(title);
            dialog.setHeaderText(header);
            dialog.getDialogPane().getButtonTypes().addAll(ButtonType.OK, ButtonType.CANCEL);

            PasswordField pwd = new PasswordField();
            VBox vbox = new VBox();
            vbox.setAlignment(Pos.CENTER_LEFT);
            vbox.setSpacing(10);
            Label label = new Label(content);
            label.setWrapText(true);
            label.setMaxWidth(450);
            vbox.getChildren().addAll(label, pwd);
            dialog.getDialogPane().setContent(vbox);
            dialog.setResultConverter(dialogButton -> {
                if (dialogButton == ButtonType.OK) {
                    return pwd.getText();
                }
                return null;
            });
            dialog.showAndWait().ifPresentOrElse(result::set, () -> result.set(null));
            countDownLatch.countDown();
        });

        try {
            countDownLatch.await();
        } catch (InterruptedException e) {
            LOGGER.error("Exception during password input dialog ", e);
            Thread.currentThread().interrupt();
        }
        return result.get();

    }

    public static <T> T select(String title, String content, List<T> choices) {
        final CountDownLatch countDownLatch = new CountDownLatch(1);
        AtomicReference<T> result = new AtomicReference<>();
        PlatformUtils.runLaterIfNotInFxThread(() -> {
            Dialog<T> dialog = new Dialog<>();
            StageHelper.enforceFloatingWindow(dialog);
            dialog.setWidth(450);
            DialogPane dialogPane = dialog.getDialogPane();
            dialog.setWidth(450);
            dialogPane.setMinHeight(Region.USE_PREF_SIZE);
            String cssPath = SettingsProvider.getInstance().getCssPath();
            if (cssPath != null) {
                dialogPane.getStylesheets().add(cssPath);
            }
            dialog.setTitle(title);
            dialog.getDialogPane().getButtonTypes().addAll(ButtonType.OK);

            ComboBox<T> comboBox = new ComboBox<>();
            comboBox.setItems(FXCollections.observableArrayList(choices));
            comboBox.getSelectionModel().selectFirst();
            VBox vbox = new VBox();
            vbox.setAlignment(Pos.CENTER_LEFT);
            vbox.setSpacing(10);
            Label label = new Label(content);
            label.setWrapText(true);
            label.setMaxWidth(450);
            vbox.getChildren().addAll(label, comboBox);
            dialog.getDialogPane().setContent(vbox);
            dialog.setResultConverter(dialogButton -> {
                if (dialogButton == ButtonType.OK) {
                    return comboBox.getSelectionModel().getSelectedItem();
                }
                return null;
            });
            dialog.showAndWait().ifPresentOrElse(result::set, () -> result.set(null));
            countDownLatch.countDown();
        });

        try {
            countDownLatch.await();
        } catch (InterruptedException e) {
            LOGGER.error("Exception during password input dialog ", e);
            Thread.currentThread().interrupt();
        }
        return result.get();
    }

}
