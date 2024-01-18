package org.correomqtt.gui.model;

import javafx.scene.paint.Paint;
import lombok.Getter;
import org.correomqtt.business.connection.ConnectionState;

import java.util.Arrays;

@Getter
public enum GuiConnectionState {

    CONNECTED(Paint.valueOf(Constants.GREEN), ConnectionState.CONNECTED),
    CONNECTING(Paint.valueOf(Constants.ORANGE), ConnectionState.CONNECTING),
    RECONNECTING(Paint.valueOf(Constants.ORANGE), ConnectionState.RECONNECTING),
    DISCONNECTING(Paint.valueOf(Constants.ORANGE), ConnectionState.DISCONNECTING),
    DISCONNECTED_GRACEFUL(Paint.valueOf(Constants.GRAY), ConnectionState.DISCONNECTED_GRACEFUL),
    DISCONNECTED_UNGRACEFUL(Paint.valueOf(Constants.RED), ConnectionState.DISCONNECTED_UNGRACEFUL);

    private final Paint iconColor;
    private final ConnectionState clientState;

    GuiConnectionState(Paint iconColor, ConnectionState clientState) {
        this.iconColor = iconColor;
        this.clientState = clientState;
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
}
