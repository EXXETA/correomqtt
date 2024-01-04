package org.correomqtt.gui.model;

import javafx.scene.paint.Paint;

public enum ConnectionState {
    CONNECTED(Paint.valueOf("green")),
    CONNECTING(Paint.valueOf("orange")),
    DISCONNECTING(Paint.valueOf("orange")),
    DISCONNECTED_GRACEFUL(Paint.valueOf("gray")),
    DISCONNECTED_UNGRACEFUL(Paint.valueOf("red"));

    private final Paint paint;

    ConnectionState(Paint paint) {
        this.paint = paint;
    }

    public Paint getIconColor() {
        return paint;
    }
}
