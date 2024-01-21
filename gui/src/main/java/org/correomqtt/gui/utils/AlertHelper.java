package org.correomqtt.gui.utils;

import javafx.application.Platform;
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
import javafx.scene.control.TextField;
import javafx.scene.layout.Region;
import javafx.scene.layout.VBox;
import javafx.stage.StageStyle;
import javafx.stage.Window;
import org.correomqtt.core.settings.SettingsManager;
import org.correomqtt.gui.theme.ThemeManager;
import org.correomqtt.gui.window.StageHelper;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import javax.inject.Inject;
import java.util.List;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicReference;

import static javafx.scene.control.Alert.AlertType.INFORMATION;
import static javafx.scene.control.Alert.AlertType.WARNING;

public class AlertHelper {

    private static final Logger LOGGER = LoggerFactory.getLogger(AlertHelper.class);
    private final SettingsManager settingsManager;
    private final ThemeManager themeManager;

    @Inject
    AlertHelper(SettingsManager settingsManager,
                ThemeManager themeManager) {
        this.settingsManager = settingsManager;
        this.themeManager = themeManager;
    }

    public void showDialog(AlertType type, String title, String content, boolean block, ButtonType buttonType) {
        final CountDownLatch countDownLatch = block ? new CountDownLatch(1) : null;
        PlatformUtils.runLaterIfNotInFxThread(() -> {
            Alert alert = new Alert(type);
            StageHelper.enforceFloatingWindow(alert);
            DialogPane dialogPane = alert.getDialogPane();
            String cssPath = themeManager.getCssPath();
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

    public boolean showConfirmationDialog(String title, String header, String content, String noButton, String yesButton) {
        final CountDownLatch countDownLatch = new CountDownLatch(1);
        AtomicBoolean result = new AtomicBoolean();
        PlatformUtils.runLaterIfNotInFxThread(() -> {
            Alert alert = new Alert(Alert.AlertType.CONFIRMATION);
            StageHelper.enforceFloatingWindow(alert);
            DialogPane dialogPane = alert.getDialogPane();
            dialogPane.setMaxWidth(450);
            String cssPath = themeManager.getCssPath();
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

    public void info(String title, String content) {
        info(title, content, false);
    }

    public void info(String title, String content, boolean block) {
        showDialog(INFORMATION, title, content, block, null);
    }

    public void info(String title, String content, boolean block, ButtonType buttonType) {
        showDialog(INFORMATION, title, content, block, buttonType);
    }

    public void warn(String title, String content) {
        warn(title, content, false);
    }

    public void warn(String title, String content, boolean block) {
        showDialog(WARNING, title, content, block, null);
    }

    public void warn(String title, String content, boolean block, ButtonType buttonType) {
        showDialog(WARNING, title, content, block, buttonType);
    }

    public boolean confirm(String title, String header, String content, String noButton, String yesButton) {
        return showConfirmationDialog(title, header, content, noButton, yesButton);
    }

    public String input(String title, String header, String content, String defaultValue) {
        final CountDownLatch countDownLatch = new CountDownLatch(1);
        AtomicReference<String> result = new AtomicReference<>();
        PlatformUtils.runLaterIfNotInFxThread(() -> {
            Dialog<String> dialog = new Dialog<>();
            StageHelper.enforceFloatingWindow(dialog);
            dialog.setWidth(450);
            DialogPane dialogPane = dialog.getDialogPane();
            dialog.setWidth(450);
            dialogPane.setMinHeight(Region.USE_PREF_SIZE);
            String cssPath = themeManager.getCssPath();
            if (cssPath != null) {
                dialogPane.getStylesheets().add(cssPath);
            }
            dialog.setTitle(title);
            dialog.setHeaderText(header);
            dialog.getDialogPane().getButtonTypes().addAll(ButtonType.OK, ButtonType.CANCEL);

            TextField textField = new TextField();
            textField.setText(defaultValue);
            VBox vbox = new VBox();
            vbox.setAlignment(Pos.CENTER_LEFT);
            vbox.setSpacing(10);
            Label label = new Label(content);
            label.setWrapText(true);
            label.setMaxWidth(450);
            vbox.getChildren().addAll(label, textField);
            dialog.getDialogPane().setContent(vbox);
            dialog.setResultConverter(dialogButton -> {
                if (dialogButton == ButtonType.OK) {
                    return textField.getText();
                }
                return null;
            });
            Platform.runLater(textField::requestFocus);
            dialog.showAndWait().ifPresentOrElse(result::set, () -> result.set(null));
            countDownLatch.countDown();
        });

        try {
            countDownLatch.await();
        } catch (InterruptedException e) {
            LOGGER.error("Exception during text input dialog ", e);
            Thread.currentThread().interrupt();
        }
        return result.get();

    }


    public String passwordInput(String title, String header, String content) {
        final CountDownLatch countDownLatch = new CountDownLatch(1);
        AtomicReference<String> result = new AtomicReference<>();
        PlatformUtils.runLaterIfNotInFxThread(() -> {
            Dialog<String> dialog = new Dialog<>();
            StageHelper.enforceFloatingWindow(dialog);
            dialog.setWidth(450);
            DialogPane dialogPane = dialog.getDialogPane();
            dialog.setWidth(450);
            dialogPane.setMinHeight(Region.USE_PREF_SIZE);
            String cssPath = themeManager.getCssPath();
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

    public <T> T select(String title, String content, List<T> choices) {
        final CountDownLatch countDownLatch = new CountDownLatch(1);
        AtomicReference<T> result = new AtomicReference<>();
        PlatformUtils.runLaterIfNotInFxThread(() -> {
            Dialog<T> dialog = new Dialog<>();
            StageHelper.enforceFloatingWindow(dialog);
            dialog.setWidth(450);
            DialogPane dialogPane = dialog.getDialogPane();
            dialog.setWidth(450);
            dialogPane.setMinHeight(Region.USE_PREF_SIZE);
            String cssPath = themeManager.getCssPath();
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

    public void unexpectedAlert(Throwable unexpectedError) {
        LOGGER.error("Unexpected Error", unexpectedError);
        warn("Unexpected Error", "Operation failed: " + unexpectedError.getMessage(), true);
    }
}
