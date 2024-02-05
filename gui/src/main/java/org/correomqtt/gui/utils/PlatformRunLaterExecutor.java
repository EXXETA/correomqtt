package org.correomqtt.gui.utils;

import javafx.application.Platform;
import org.correomqtt.di.TaskToFrontendPush;

@SuppressWarnings("unused")
public class PlatformRunLaterExecutor implements TaskToFrontendPush {
    @Override
    public void pushToFrontend(Runnable runnable) {
        Platform.runLater(runnable);
    }
}
