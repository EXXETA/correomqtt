package org.correomqtt.gui.views.connections;

import javafx.event.ActionEvent;
import javafx.fxml.FXML;
import javafx.scene.control.Button;
import javafx.scene.control.Label;
import javafx.scene.control.ToggleButton;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.HBox;
import org.correomqtt.business.connection.ConnectTask;
import org.correomqtt.business.connection.ConnectionState;
import org.correomqtt.business.connection.ConnectionStateChangedEvent;
import org.correomqtt.business.connection.DisconnectTask;
import org.correomqtt.business.connection.ReconnectTask;
import org.correomqtt.business.eventbus.EventBus;
import org.correomqtt.business.eventbus.Subscribe;
import org.correomqtt.business.fileprovider.SettingsProvider;
import org.correomqtt.business.model.ConnectionConfigDTO;
import org.correomqtt.business.utils.ConnectionHolder;
import org.correomqtt.gui.controls.IconLabel;
import org.correomqtt.gui.model.GuiConnectionState;
import org.correomqtt.gui.views.LoaderResult;
import org.correomqtt.plugin.manager.PluginManager;
import org.correomqtt.plugin.spi.MainToolbarHook;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.ResourceBundle;

import static org.correomqtt.business.connection.ConnectionState.CONNECTED;

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


    private void updateBrokerInfo() {
        ConnectionConfigDTO config = ConnectionHolder.getInstance().getConfig(getConnectionId());
        if (config != null) {
            brokerInfo.setText(config.getHostAndPort());
        }
    }


    @SuppressWarnings("unused")
    public void onConnectionStateChanged(@Subscribe ConnectionStateChangedEvent event) {

        ConnectionState state = event.getState();

        disconnectBtn.setVisible(false);
        disconnectBtn.setManaged(false);
        disconnectBtn.setDisable(true);
        reconnectBtn.setVisible(false);
        reconnectBtn.setManaged(false);
        reconnectBtn.setDisable(true);
        connectBtn.setVisible(false);
        connectBtn.setManaged(false);
        connectBtn.setDisable(true);

        switch (state) {
            case CONNECTED -> {
                disconnectBtn.setVisible(true);
                disconnectBtn.setManaged(true);
                disconnectBtn.setDisable(false);
                statusInfo.setText(resources.getString("controlBarControllerConnected"));
            }
            case CONNECTING -> {
                connectBtn.setVisible(true);
                connectBtn.setManaged(true);
                connectBtn.setDisable(true);
                statusInfo.setText(resources.getString("controlBarControllerConnecting"));
            }
            case RECONNECTING -> {
                reconnectBtn.setVisible(true);
                reconnectBtn.setManaged(true);
                reconnectBtn.setDisable(true);
                statusInfo.setText(resources.getString("controlBarControllerReconnecting") + " " + event.getRetries() + "/" + event.getMaxRetries());
            }
            case DISCONNECTING -> {
                disconnectBtn.setVisible(true);
                disconnectBtn.setManaged(true);
                disconnectBtn.setDisable(true);
                statusInfo.setText(resources.getString("controlBarControllerDisconnecting"));
            }
            case DISCONNECTED_GRACEFUL -> {
                connectBtn.setVisible(true);
                connectBtn.setManaged(true);
                connectBtn.setDisable(false);
                statusInfo.setText(resources.getString("controlBarControllerDisconnected"));
            }
            case DISCONNECTED_UNGRACEFUL -> {
                reconnectBtn.setVisible(true);
                reconnectBtn.setManaged(true);
                reconnectBtn.setDisable(false);
                statusInfo.setText("Failed.");
            }
        }


        updateBrokerInfo();

        GuiConnectionState guiState = GuiConnectionState.of(state);

        delegate.setConnectionState(guiState);
        statusLabel.setIconColor(guiState.getIconColor());

        controlViewPSButton.setDisable(state != CONNECTED);
        controlViewPButton.setDisable(state != CONNECTED);
        controlViewSButton.setDisable(state != CONNECTED);


    }


    public void cleanUp() {
        EventBus.unregister(this);
    }
}
