package org.correomqtt.gui.helper;

import javafx.application.Platform;
import javafx.scene.control.Alert;
import javafx.scene.control.Alert.AlertType;
import javafx.scene.control.ButtonType;
import javafx.scene.control.DialogPane;
import org.correomqtt.business.services.ConfigService;
import org.correomqtt.gui.utils.PlatformUtils;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.concurrent.CountDownLatch;
import java.util.concurrent.atomic.AtomicBoolean;

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
            DialogPane dialogPane = alert.getDialogPane();
            String cssPath = ConfigService.getInstance().getCssPath(false);
            if (cssPath != null) {
                dialogPane.getStylesheets().add(cssPath);
            }
            alert.setTitle(title);
            alert.setHeaderText(null);
            alert.setContentText(content);
            if (buttonType != null) {
                alert.getButtonTypes().setAll(buttonType);
            }

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
}
