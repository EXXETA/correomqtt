package org.correomqtt.gui.controller;

import org.correomqtt.business.dispatcher.ConnectionLifecycleDispatcher;
import org.correomqtt.business.dispatcher.ConnectionLifecycleObserver;
import org.correomqtt.business.exception.CorreoMqttException;
import org.correomqtt.business.model.ConnectionConfigDTO;
import org.correomqtt.business.utils.ConnectionHolder;
import org.correomqtt.gui.business.TaskFactory;
import org.correomqtt.gui.model.ConnectionState;
import javafx.application.Platform;
import javafx.event.ActionEvent;
import javafx.fxml.FXML;
import javafx.scene.control.Button;
import javafx.scene.control.Label;
import javafx.scene.control.ToggleButton;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.HBox;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.ResourceBundle;
import java.util.concurrent.atomic.AtomicInteger;

public class ControlBarController extends BaseConnectionController implements ConnectionLifecycleObserver {
    private static final Logger LOGGER = LoggerFactory.getLogger(ControlBarController.class);
    private final ControlBarDelegate delegate;
    @FXML
    public AnchorPane mainViewHBoxAnchorPane;
    @FXML
    public HBox controllViewMainViewHBox;
    @FXML
    public Button connectBtn;
    @FXML
    public Button disconnectBtn;
    @FXML
    public ToggleButton controlViewPButton;
    @FXML
    public ToggleButton controlViewPSButton;
    @FXML
    public ToggleButton controlViewSButton;
    @FXML
    public Label statusLabel;
    @FXML
    public Label statusInfo;
    @FXML
    public Label brokerInfo;
    @FXML
    private Button sysButton;
    @FXML
    private ResourceBundle resources;

    boolean gracefulDisconnenct = false;

    public ControlBarController(String connectionId, ControlBarDelegate delegate) {
        super(connectionId);
        this.delegate = delegate;
        ConnectionLifecycleDispatcher.getInstance().addObserver(this);
    }

    static LoaderResult<ControlBarController> load(String connectionId, ControlBarDelegate delegate) {
        return load(ControlBarController.class, "controlBarView.fxml",
                    () -> new ControlBarController(connectionId, delegate)
        );
    }

    @FXML
    public void initialize() {
        controlViewPSButton.setSelected(true);
        brokerInfo.setText("");
        disconnectBtn.setVisible(false);
        disconnectBtn.setManaged(false);
    }

    @FXML
    public void onClickConnect(ActionEvent actionEvent) {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Connect in control bar clicked for connection: {}", getConnectionId());
        }

        TaskFactory.connect(getConnectionId());
    }

    @FXML
    public void onClickDisconnect() {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Disconnect in control bar clicked for connection: {}", getConnectionId());
        }

        gracefulDisconnenct = true;
        TaskFactory.disconnect(getConnectionId());
    }

    @FXML
    public void onClickP() {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Show only publish clicked: {}", getConnectionId());
        }

        delegate.setLayout(true,false);
        controlViewPButton.setSelected(true);
        controlViewPSButton.setSelected(false);
        controlViewSButton.setSelected(false);
    }

    @FXML
    public void onClickPS() {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Show publish AND subscribe clicked: {}", getConnectionId());
        }

        delegate.setLayout(true,true);
        controlViewPButton.setSelected(false);
        controlViewPSButton.setSelected(true);
        controlViewSButton.setSelected(false);
    }

    @FXML
    public void onClickS() {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Show only subscribe clicked: {}", getConnectionId());
        }

        delegate.setLayout(false,true);
        controlViewPButton.setSelected(false);
        controlViewPSButton.setSelected(false);
        controlViewSButton.setSelected(true);
    }

    @FXML
    public void onSysClicked() {
        SysTopicViewController.showAsDialog(getConnectionId());
    }

    private void setGuiDisconnected() {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Set gui to disconnected state: {}", getConnectionId());
        }

        statusLabel.getStyleClass().clear();

        if (gracefulDisconnenct) {
            statusLabel.getStyleClass().add("grayStatus");
            delegate.setConnectionState(ConnectionState.DISCONNECTED_GRACEFUL);
            gracefulDisconnenct = false;
        } else {
            statusLabel.getStyleClass().add("redStatus");
            delegate.setConnectionState(ConnectionState.DISCONNECTED_UNGRACEFUL);
        }

        statusInfo.setVisible(true);
        statusInfo.setManaged(true);
        updateBrokerInfo();
        connectBtn.setVisible(true);
        connectBtn.setManaged(true);
        connectBtn.setDisable(false);
        disconnectBtn.setVisible(false);
        disconnectBtn.setManaged(false);
        controlViewPSButton.setDisable(true);
        controlViewPButton.setDisable(true);
        controlViewSButton.setDisable(true);
        sysButton.setDisable(true);
    }

    private void updateBrokerInfo() {
        ConnectionConfigDTO config = ConnectionHolder.getInstance().getConfig(getConnectionId());
        if (config != null) {
            brokerInfo.setText(config.getHostAndPort());
        }
    }

    private void setGuiConnected() {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Set gui to connected state: {}", getConnectionId());
        }

        statusInfo.setVisible(false);
        statusInfo.setManaged(false);
        updateBrokerInfo();
        connectBtn.setVisible(false);
        connectBtn.setManaged(false);
        disconnectBtn.setVisible(true);
        disconnectBtn.setManaged(true);
        disconnectBtn.setDisable(false);
        statusLabel.getStyleClass().clear();
        statusLabel.getStyleClass().add("greenStatus");
        delegate.setConnectionState(ConnectionState.CONNECTED);
        controlViewPSButton.setDisable(false);
        controlViewPButton.setDisable(false);
        controlViewSButton.setDisable(false);
        sysButton.setDisable(false);
    }

    private void setGuiConnecting() {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Set gui to connecting state: {}", getConnectionId());
        }

        statusInfo.setVisible(true);
        statusInfo.setManaged(true);
        updateBrokerInfo();
        connectBtn.setDisable(true);
        statusLabel.getStyleClass().clear();
        statusLabel.getStyleClass().add("yellowStatus");
        delegate.setConnectionState(ConnectionState.CONNECTING);
        controlViewPSButton.setDisable(true);
        controlViewPButton.setDisable(true);
        controlViewSButton.setDisable(true);
        sysButton.setDisable(true);
    }

    private void setGuiDisconnecting() {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Set gui to disconnecting state: {}", getConnectionId());
        }

        statusInfo.setVisible(true);
        statusInfo.setManaged(true);
        updateBrokerInfo();
        disconnectBtn.setDisable(true);
        statusLabel.getStyleClass().clear();
        statusLabel.getStyleClass().add("yellowStatus");
        delegate.setConnectionState(ConnectionState.DISCONNECTING);
        controlViewPSButton.setDisable(true);
        controlViewPButton.setDisable(true);
        controlViewSButton.setDisable(true);
        sysButton.setDisable(true);
    }

    @Override
    public void onDisconnectFromConnectionDeleted(String connectionId) {
        // do nothing
    }

    @Override
    public void onConnect() {
        Platform.runLater(() -> {
            statusInfo.setText(resources.getString("controlBarControllerConnected"));
            setGuiConnected();
        });
    }

    @Override
    public void onConnectRunning() {
        Platform.runLater(() -> {
            statusInfo.setText(resources.getString("controlBarControllerConnecting"));
            setGuiConnecting();
        });
    }

    @Override
    public void onConnectionFailed(Throwable e) {
        String msg;
        if (e instanceof CorreoMqttException) {
            msg = ((CorreoMqttException) e).getInfo();
        } else {
            msg = e.getClass().toString() + ":" + e.getMessage();
        }
        Platform.runLater(() -> {
            statusInfo.setText(msg);
            setGuiDisconnected();
        });
    }

    @Override
    public void onConnectionCanceled() {
        Platform.runLater(() -> {
            statusInfo.setText(resources.getString("controlBarControllerConnectCanceled"));
            setGuiConnecting();
        });
    }

    @Override
    public void onConnectionLost() {
        Platform.runLater(() -> {
            statusInfo.setText(resources.getString("controlBarControllerConnectLost"));
            setGuiConnecting();
        });
    }

    @Override
    public void onDisconnect() {
        Platform.runLater(() -> {
            statusInfo.setText(resources.getString("controlBarControllerDisconnected"));
            setGuiDisconnected();
        });
    }

    @Override
    public void onDisconnectFailed(Throwable exception) {
        Platform.runLater(() -> {
            statusInfo.setText(resources.getString("controlBarControllerDisconnectFailed"));
            setGuiConnected();
        });
    }

    @Override
    public void onDisconnectRunning() {
        Platform.runLater(() -> {
            statusInfo.setText(resources.getString("controlBarControllerDisconnecting"));
            setGuiDisconnecting();
        });
    }

    @Override
    public void onConnectionReconnected() {
        Platform.runLater(() -> {
            statusInfo.setText(resources.getString("controlBarControllerConnected"));
            setGuiConnected();
        });
    }

    @Override
    public void onReconnectFailed(AtomicInteger triedReconnects, int maxReconnects) {
        Platform.runLater(() -> {
            statusInfo.setText(resources.getString("controlBarControllerReconnecting") + " " + triedReconnects + "/" + maxReconnects);
            setGuiConnecting();
        });
    }
}
