package org.correomqtt.gui.window;

import javafx.scene.control.Dialog;
import javafx.stage.Stage;
import javafx.stage.Window;
import javafx.stage.StageStyle;

public class StageHelper {

    private StageHelper() {
        // empty constructor
    }

    public static void enforceFloatingWindow(Window window) {
        enforceFloatingWindow((Stage) window);
    }

    public static <T> void enforceFloatingWindow(Dialog<T> dialog) {
        enforceFloatingWindow((Stage) dialog.getDialogPane().getScene().getWindow());
    }

    public static void enforceFloatingWindow(Stage stage) {

        String xdgCurrentDesktop = System.getenv("XDG_CURRENT_DESKTOP");

        if ("i3".equals(xdgCurrentDesktop) || "sway".equals(xdgCurrentDesktop)) {
            stage.setTitle("CorreoMQTT");
            stage.initStyle(StageStyle.UTILITY);
        } else {
            stage.initStyle(StageStyle.UNDECORATED);
        }
    }
}
