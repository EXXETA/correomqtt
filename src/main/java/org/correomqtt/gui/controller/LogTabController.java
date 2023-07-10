package org.correomqtt.gui.controller;

import org.correomqtt.business.dispatcher.ConnectionLifecycleObserver;
import org.correomqtt.business.dispatcher.LogDispatcher;
import org.correomqtt.business.dispatcher.LogObserver;
import javafx.fxml.FXML;
import javafx.scene.control.Button;
import javafx.scene.control.TextArea;
import javafx.scene.layout.AnchorPane;

import java.util.concurrent.atomic.AtomicInteger;

public class LogTabController extends BaseController implements LogObserver, ConnectionLifecycleObserver {
    @FXML
    public AnchorPane logViewAnchor;
    @FXML
    public TextArea logTextArea;
    @FXML
    private Button trashButton;

    public LogTabController() {
        LogDispatcher.getInstance().addObserver(this);
    }

    public static LoaderResult<LogTabController> load() {
        return load(LogTabController.class, "logView.fxml");
    }

    @FXML
    private void initialize() {
        trashButton.setOnAction(event -> logTextArea.setText(""));
    }

    @Override
    public void updateLog(String message) {
        logTextArea.appendText(message);
    }

    @Override
    public String getConnectionId() {
        return null;
    }

    @Override
    public void onDisconnectFromConnectionDeleted(String connectionId) {

    }

    @Override
    public void onConnect() {

    }

    @Override
    public void onConnectRunning() {

    }

    @Override
    public void onConnectionFailed(Throwable message) {

    }

    @Override
    public void onConnectionLost() {

    }

    @Override
    public void onDisconnect() {

    }

    @Override
    public void onDisconnectFailed(Throwable exception) {

    }

    @Override
    public void onDisconnectRunning() {

    }

    @Override
    public void onConnectionReconnected() {

    }

    @Override
    public void onReconnectFailed(AtomicInteger triedReconnects, int maxReconnects) {

    }

    @Override
    public void onCleanUp(String connectinId) {
        LogDispatcher.getInstance().removeObserver(this);
    }
}
