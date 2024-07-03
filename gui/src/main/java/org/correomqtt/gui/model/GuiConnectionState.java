package org.correomqtt.gui.model;

import javafx.collections.ObservableList;
import javafx.css.Styleable;
import javafx.scene.paint.Paint;
import lombok.Getter;
import org.correomqtt.core.connection.ConnectionState;

import java.util.Arrays;

@Getter
public enum GuiConnectionState {

    CONNECTED(Paint.valueOf(Constants.GREEN), ConnectionState.CONNECTED, "successIcon"),
    CONNECTING(Paint.valueOf(Constants.ORANGE), ConnectionState.CONNECTING, "warnIcon"),
    RECONNECTING(Paint.valueOf(Constants.ORANGE), ConnectionState.RECONNECTING, "warnIcon"),
    DISCONNECTING(Paint.valueOf(Constants.ORANGE), ConnectionState.DISCONNECTING, "warnIcon"),
    DISCONNECTED_GRACEFUL(Paint.valueOf(Constants.GRAY), ConnectionState.DISCONNECTED_GRACEFUL, "ignoreIcon"),
    DISCONNECTED_UNGRACEFUL(Paint.valueOf(Constants.RED), ConnectionState.DISCONNECTED_UNGRACEFUL, "failIcon");

    private final Paint iconColor;
    private final ConnectionState clientState;
    private final String cssClass;

    GuiConnectionState(Paint iconColor, ConnectionState clientState, String cssClass) {
        this.iconColor = iconColor;
        this.clientState = clientState;
        this.cssClass = cssClass;
    }

    public static GuiConnectionState of(ConnectionState state) {
        return Arrays.stream(values())
                .filter(gcs -> gcs.clientState == state)
                .findFirst()
                .orElse(DISCONNECTED_GRACEFUL);
    }

    private static class Constants {
        public static final String ORANGE = "orange";
        public static final String GREEN = "green";
        public static final String GRAY = "gray";
        public static final String RED = "red";
    }

    public void applyCssClass(Styleable styleable) {
        ObservableList<String> cssClasses = styleable.getStyleClass();
        Arrays.stream(GuiConnectionState.values()).forEach(
                g -> cssClasses.remove(g.getCssClass())
        );
        cssClasses.add(this.getCssClass());
    }
}
