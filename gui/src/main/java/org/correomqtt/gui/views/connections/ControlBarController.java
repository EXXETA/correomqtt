package org.correomqtt.gui.views.connections;

import dagger.assisted.Assisted;
import dagger.assisted.AssistedFactory;
import dagger.assisted.AssistedInject;
import javafx.fxml.FXML;
import javafx.scene.control.Button;
import javafx.scene.control.Label;
import javafx.scene.control.ToggleButton;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.HBox;
import org.correomqtt.core.CoreManager;
import org.correomqtt.core.connection.ConnectionLifecycleTaskFactories;
import org.correomqtt.core.connection.ConnectionState;
import org.correomqtt.core.connection.ConnectionStateChangedEvent;
import org.correomqtt.core.eventbus.EventBus;
import org.correomqtt.core.eventbus.Subscribe;
import org.correomqtt.core.model.ConnectionConfigDTO;
import org.correomqtt.gui.controls.IconLabel;
import org.correomqtt.gui.model.GuiConnectionState;
import org.correomqtt.gui.plugin.spi.MainToolbarHook;
import org.correomqtt.gui.theme.ThemeManager;
import org.correomqtt.gui.views.LoaderResult;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.ResourceBundle;

import static org.correomqtt.core.connection.ConnectionState.CONNECTED;

public class ControlBarController extends BaseConnectionController {
    private static final Logger LOGGER = LoggerFactory.getLogger(ControlBarController.class);

    private final ConnectionLifecycleTaskFactories connectionLifecycleTaskFactories;
    private final ControlBarDelegate delegate;

    @FXML
    private AnchorPane mainViewHBoxAnchorPane;

    @FXML
    private HBox controllViewButtonHBox;

    @FXML
    private Button connectBtn;

    @FXML
    private Button disconnectBtn;

    @FXML
    private Button reconnectBtn;

    @FXML
    private ToggleButton controlViewPButton;

    @FXML
    private ToggleButton controlViewPSButton;

    @FXML
    private ToggleButton controlViewSButton;

    @FXML
    private IconLabel statusLabel;

    @FXML
    private Label statusInfo;

    @FXML
    private Label brokerInfo;

    @FXML
    private ResourceBundle resources;

    private ConnectionConfigDTO connectionConfigDTO;

    @AssistedFactory

    public interface Factory {
        ControlBarController create(String connectionId,
                                    ControlBarDelegate delegate);

    }

    @AssistedInject
    public ControlBarController(CoreManager coreManager,
                                ConnectionLifecycleTaskFactories connectionLifecycleTaskFactories,
                                ThemeManager themeManager,
                                @Assisted String connectionId,
                                @Assisted ControlBarDelegate delegate) {
        super(coreManager, themeManager, connectionId);
        this.connectionLifecycleTaskFactories = connectionLifecycleTaskFactories;
        this.delegate = delegate;
        EventBus.register(this);
    }

    LoaderResult<ControlBarController> load() {
        return load(ControlBarController.class, "controlBarView.fxml",
                () -> this
        );
    }

    @FXML
    private void initialize() {
        coreManager.getSettingsManager().getConnectionConfigs().stream()
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

        coreManager.getPluginManager().getExtensions(MainToolbarHook.class).forEach(p -> p.onInstantiateMainToolbar(getConnectionId(), controllViewButtonHBox, indexToInsert));
    }

    @FXML
    private void onClickReconnect() {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Reconnect in control bar clicked for connection: {}", getConnectionId());
        }
        connectionLifecycleTaskFactories.getReconnectFactory().create(getConnectionId()).run();
    }

    @FXML
    private void onClickConnect() {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Connect in control bar clicked for connection: {}", getConnectionId());
        }
        connectionLifecycleTaskFactories.getConnectFactory().create(getConnectionId()).run();
    }

    @FXML
    private void onClickDisconnect() {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Disconnect in control bar clicked for connection: {}", getConnectionId());
        }
        connectionLifecycleTaskFactories.getDisconnectFactory().create(getConnectionId()).run();
    }

    @FXML
    private void onClickP() {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Show only publish clicked: {}", getConnectionId());
        }

        delegate.setLayout(true, false);
        controlViewPButton.setSelected(true);
        controlViewPSButton.setSelected(false);
        controlViewSButton.setSelected(false);
    }

    @FXML
    private void onClickPS() {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Show publish AND subscribe clicked: {}", getConnectionId());
        }

        delegate.setLayout(true, true);
        controlViewPButton.setSelected(false);
        controlViewPSButton.setSelected(true);
        controlViewSButton.setSelected(false);
    }

    @FXML
    private void onClickS() {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Show only subscribe clicked: {}", getConnectionId());
        }

        delegate.setLayout(false, true);
        controlViewPButton.setSelected(false);
        controlViewPSButton.setSelected(false);
        controlViewSButton.setSelected(true);
    }

    @FXML
    private void saveUISettings() {
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
    private void resetUISettings() {
        controlViewPButton.setSelected(false);
        controlViewPSButton.setSelected(true);
        controlViewSButton.setSelected(false);

        delegate.resetConnectionUISettings();
    }


    private void updateBrokerInfo() {
        ConnectionConfigDTO config = coreManager.getConnectionManager().getConfig(getConnectionId());
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
