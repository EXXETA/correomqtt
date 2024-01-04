package org.correomqtt.gui.views.connections;

import javafx.application.Platform;
import javafx.event.ActionEvent;
import javafx.fxml.FXML;
import javafx.scene.control.Button;
import javafx.scene.control.Label;
import javafx.scene.control.ToggleButton;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.HBox;
import org.correomqtt.business.connection.AutomaticReconnectEvent;
import org.correomqtt.business.connection.AutomaticReconnectFailedEvent;
import org.correomqtt.business.connection.ConnectEvent;
import org.correomqtt.business.connection.ConnectFailedEvent;
import org.correomqtt.business.connection.ConnectStartedEvent;
import org.correomqtt.business.connection.ConnectTask;
import org.correomqtt.business.connection.DisconnectEvent;
import org.correomqtt.business.connection.DisconnectFailedEvent;
import org.correomqtt.business.connection.DisconnectStartedEvent;
import org.correomqtt.business.connection.DisconnectTask;
import org.correomqtt.business.connection.ReconnectTask;
import org.correomqtt.business.eventbus.EventBus;
import org.correomqtt.business.eventbus.Subscribe;
import org.correomqtt.gui.controls.IconLabel;
import org.correomqtt.gui.views.LoaderResult;
import org.correomqtt.plugin.spi.MainToolbarHook;
import org.correomqtt.business.exception.CorreoMqttException;
import org.correomqtt.business.model.ConnectionConfigDTO;
import org.correomqtt.business.fileprovider.SettingsProvider;
import org.correomqtt.business.utils.ConnectionHolder;
import org.correomqtt.gui.model.ConnectionState;
import org.correomqtt.plugin.manager.PluginManager;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.ResourceBundle;

public class ControlBarController extends BaseConnectionController {
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
    public IconLabel statusLabel;

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
        EventBus.register(this);
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
        new ReconnectTask(getConnectionId()).run();
    }

    @FXML
    public void onClickConnect(ActionEvent actionEvent) {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Connect in control bar clicked for connection: {}", getConnectionId());
        }

        new ConnectTask(getConnectionId()).run();
    }

    @FXML
    public void onClickDisconnect() {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Disconnect in control bar clicked for connection: {}", getConnectionId());
        }

        gracefulDisconnenct = true;
        new DisconnectTask(getConnectionId()).run();
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


        if (gracefulDisconnenct) {
            delegate.setConnectionState(ConnectionState.DISCONNECTED_GRACEFUL);
            statusLabel.setIconColor(ConnectionState.DISCONNECTED_GRACEFUL.getIconColor());
            gracefulDisconnenct = false;
            connectBtn.setVisible(true);
            connectBtn.setManaged(true);
        } else {
            reconnectBtn.setVisible(true);
            reconnectBtn.setManaged(true);
            connectBtn.setVisible(false);
            connectBtn.setManaged(false);
            delegate.setConnectionState(ConnectionState.DISCONNECTED_UNGRACEFUL);
            statusLabel.setIconColor(ConnectionState.DISCONNECTED_UNGRACEFUL.getIconColor());
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
        delegate.setConnectionState(ConnectionState.CONNECTED);
        statusLabel.setIconColor(ConnectionState.CONNECTED.getIconColor());
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
        delegate.setConnectionState(ConnectionState.CONNECTING);
        statusLabel.setIconColor(ConnectionState.CONNECTING.getIconColor());
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
        delegate.setConnectionState(ConnectionState.DISCONNECTING);
        statusLabel.setIconColor(ConnectionState.DISCONNECTING.getIconColor());
        controlViewPSButton.setDisable(true);
        controlViewPButton.setDisable(true);
        controlViewSButton.setDisable(true);

    }

    @Subscribe(ConnectEvent.class)
    public void onConnect() {
        Platform.runLater(() -> {
            statusInfo.setText(resources.getString("controlBarControllerConnected"));
            setGuiConnected();
        });
    }

    @Subscribe(ConnectStartedEvent.class)
    public void onConnectStarted() {
        Platform.runLater(() -> {
            statusInfo.setText(resources.getString("controlBarControllerConnecting"));
            setGuiConnecting();
        });
    }

    public void onConnectionFailed(@Subscribe ConnectFailedEvent event) {
        String msg;
        Throwable e = event.getThrowable();
        if (e instanceof CorreoMqttException correoMqttException) {
            msg = correoMqttException.getInfo();
        } else {
            msg = e.getClass().toString() + ":" + e.getMessage();
        }
        Platform.runLater(() -> {
            statusInfo.setText(msg);
            setGuiDisconnected();
        });
    }

    @Subscribe(DisconnectEvent.class)
    public void onDisconnect() {
        Platform.runLater(() -> {
            statusInfo.setText(resources.getString("controlBarControllerDisconnected"));
            setGuiDisconnected();
        });
    }

    @Subscribe(DisconnectFailedEvent.class)
    public void onDisconnectFailed() {
        Platform.runLater(() -> {
            statusInfo.setText(resources.getString("controlBarControllerDisconnectFailed"));
            setGuiConnected();
        });
    }

    @Subscribe(DisconnectStartedEvent.class)
    public void onDisconnectStarted() {
        Platform.runLater(() -> {
            statusInfo.setText(resources.getString("controlBarControllerDisconnecting"));
            setGuiDisconnecting();
        });
    }

    @Subscribe(AutomaticReconnectEvent.class)
    public void onConnectionReconnected() {
        Platform.runLater(() -> {
            statusInfo.setText(resources.getString("controlBarControllerConnected"));
            setGuiConnected();
        });
    }

    public void onAutomaticReconnectFailed(@Subscribe AutomaticReconnectFailedEvent event) {
        Platform.runLater(() -> {
            statusInfo.setText(resources.getString("controlBarControllerReconnecting") + " " + event.getTriedConnects() + "/" + event.getMaxConnects());
            setGuiConnecting();
        });
    }

    public void cleanUp() {
        EventBus.unregister(this);
    }
}
