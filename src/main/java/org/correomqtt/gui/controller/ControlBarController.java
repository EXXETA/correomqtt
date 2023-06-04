package org.correomqtt.gui.controller;

import javafx.application.Platform;
import javafx.event.ActionEvent;
import javafx.fxml.FXML;
import javafx.scene.control.Button;
import javafx.scene.control.Label;
import javafx.scene.control.ToggleButton;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.HBox;
import org.correomqtt.plugin.spi.MainToolbarHook;
import org.correomqtt.business.dispatcher.ConnectionLifecycleDispatcher;
import org.correomqtt.business.dispatcher.ConnectionLifecycleObserver;
import org.correomqtt.business.exception.CorreoMqttException;
import org.correomqtt.business.model.ConnectionConfigDTO;
import org.correomqtt.business.provider.SettingsProvider;
import org.correomqtt.business.utils.ConnectionHolder;
import org.correomqtt.gui.business.MessageTaskFactory;
import org.correomqtt.gui.model.ConnectionState;
import org.correomqtt.plugin.manager.PluginManager;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.ResourceBundle;
import java.util.concurrent.atomic.AtomicInteger;

public class ControlBarController extends BaseConnectionController implements ConnectionLifecycleObserver {
    private static final Logger LOGGER = LoggerFactory.getLogger(ControlBarController.class);

    private final ControlBarDelegate delegate;

    private final PluginManager pluginSystem = PluginManager.getInstance();

    @FXML
    public AnchorPane mainViewHBoxAnchorPane;

    @FXML
    public HBox controllViewButtonHBox;

    @FXML
    public Button connectBtn;

    @FXML
    public Button disconnectBtn;

    @FXML
    public Button reconnectBtn;

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
    private ResourceBundle resources;

    boolean gracefulDisconnenct = false;
    private ConnectionConfigDTO connectionConfigDTO;

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
        SettingsProvider.getInstance().getConnectionConfigs().stream()
                .filter(c -> c.getId().equals(getConnectionId()))
                .findFirst()
                .ifPresent(c -> {
                    connectionConfigDTO = c;
                    if (connectionConfigDTO.getConnectionUISettings().isShowPublish() && c.getConnectionUISettings().isShowSubscribe()) {
                        controlViewPSButton.setSelected(true);
                    } else if (c.getConnectionUISettings().isShowSubscribe()) {
                        controlViewSButton.setSelected(true);
                    } else {
                        controlViewPButton.setSelected(true);
                    }
                });
        brokerInfo.setText("");
        disconnectBtn.setVisible(false);
        disconnectBtn.setManaged(false);
        reconnectBtn.setVisible(false);
        reconnectBtn.setManaged(false);

        int indexToInsert = controllViewButtonHBox.getChildrenUnmodifiable().indexOf(controlViewSButton) + 1;

        pluginSystem.getExtensions(MainToolbarHook.class).forEach(p -> p.onInstantiateMainToolbar(getConnectionId(), controllViewButtonHBox, indexToInsert));
    }

    @FXML
    public void onClickReconnect(ActionEvent actionEvent) {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Reconnect in control bar clicked for connection: {}", getConnectionId());
        }

        TaskFactory.reconnect(getConnectionId());
    }

    @FXML
    public void onClickConnect(ActionEvent actionEvent) {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Connect in control bar clicked for connection: {}", getConnectionId());
        }

        MessageTaskFactory.connect(getConnectionId());
    }

    @FXML
    public void onClickDisconnect() {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Disconnect in control bar clicked for connection: {}", getConnectionId());
        }

        gracefulDisconnenct = true;
        MessageTaskFactory.disconnect(getConnectionId());
    }

    @FXML
    public void onClickP() {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Show only publish clicked: {}", getConnectionId());
        }

        delegate.setLayout(true, false);
        controlViewPButton.setSelected(true);
        controlViewPSButton.setSelected(false);
        controlViewSButton.setSelected(false);
    }

    @FXML
    public void onClickPS() {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Show publish AND subscribe clicked: {}", getConnectionId());
        }

        delegate.setLayout(true, true);
        controlViewPButton.setSelected(false);
        controlViewPSButton.setSelected(true);
        controlViewSButton.setSelected(false);
    }

    @FXML
    public void onClickS() {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Show only subscribe clicked: {}", getConnectionId());
        }

        delegate.setLayout(false, true);
        controlViewPButton.setSelected(false);
        controlViewPSButton.setSelected(false);
        controlViewSButton.setSelected(true);
    }

    @FXML
    public void saveUISettings() {
        if (controlViewPButton.isSelected()) {
            connectionConfigDTO.getConnectionUISettings().setShowPublish(true);
            connectionConfigDTO.getConnectionUISettings().setShowSubscribe(false);
        } else if (controlViewSButton.isSelected()) {
            connectionConfigDTO.getConnectionUISettings().setShowPublish(false);
            connectionConfigDTO.getConnectionUISettings().setShowSubscribe(true);
        } else {
            connectionConfigDTO.getConnectionUISettings().setShowPublish(true);
            connectionConfigDTO.getConnectionUISettings().setShowSubscribe(true);
        }

        delegate.saveConnectionUISettings();
    }

    @FXML
    public void resetUISettings() {
        controlViewPButton.setSelected(false);
        controlViewPSButton.setSelected(true);
        controlViewSButton.setSelected(false);

        delegate.resetConnectionUISettings();
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
            connectBtn.setVisible(true);
            connectBtn.setManaged(true);
        } else {
            reconnectBtn.setVisible(true);
            reconnectBtn.setManaged(true);
            connectBtn.setVisible(false);
            connectBtn.setManaged(false);
            statusLabel.getStyleClass().add("redStatus");
            delegate.setConnectionState(ConnectionState.DISCONNECTED_UNGRACEFUL);
        }

        statusInfo.setVisible(true);
        statusInfo.setManaged(true);
        updateBrokerInfo();
        connectBtn.setDisable(false);
        disconnectBtn.setVisible(false);
        disconnectBtn.setManaged(false);
        controlViewPSButton.setDisable(true);
        controlViewPButton.setDisable(true);
        controlViewSButton.setDisable(true);

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
        reconnectBtn.setVisible(false);
        reconnectBtn.setManaged(false);
        disconnectBtn.setVisible(true);
        disconnectBtn.setManaged(true);
        disconnectBtn.setDisable(false);
        statusLabel.getStyleClass().clear();
        statusLabel.getStyleClass().add("greenStatus");
        delegate.setConnectionState(ConnectionState.CONNECTED);
        controlViewPSButton.setDisable(false);
        controlViewPButton.setDisable(false);
        controlViewSButton.setDisable(false);

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
