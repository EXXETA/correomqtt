package org.correomqtt.gui.cell;

import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.FXMLLoader;
import javafx.scene.control.Label;
import javafx.scene.control.ListCell;
import javafx.scene.control.ListView;
import javafx.scene.layout.Pane;
import org.correomqtt.business.dispatcher.ConnectionLifecycleDispatcher;
import org.correomqtt.business.dispatcher.ConnectionLifecycleObserver;
import org.correomqtt.business.model.CorreoMqttVersion;
import org.correomqtt.business.model.Lwt;
import org.correomqtt.business.model.Proxy;
import org.correomqtt.business.model.TlsSsl;
import org.correomqtt.business.mqtt.CorreoMqttClient;
import org.correomqtt.business.provider.SettingsProvider;
import org.correomqtt.business.utils.ConnectionHolder;
import org.correomqtt.gui.model.ConnectionPropertiesDTO;
import org.correomqtt.gui.transformer.ConnectionTransformer;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.ResourceBundle;
import java.util.concurrent.atomic.AtomicInteger;

public class ConnectionCellButton extends ListCell<ConnectionPropertiesDTO> implements ConnectionLifecycleObserver {

    private static final Logger LOGGER = LoggerFactory.getLogger(ConnectionCellButton.class);
    private final ListView<ConnectionPropertiesDTO> listView;

    @FXML
    private Pane mainNode;
    @FXML
    private Label nameLabel;
    @FXML
    private ResourceBundle resources;
    @FXML
    private Label statusLabel;

    private FXMLLoader loader;

    private ConnectionPropertiesDTO connectionDTO;

    @FXML
    public void initialize() {
        ConnectionLifecycleDispatcher.getInstance().addObserver(this);

    }

    public ConnectionCellButton(ListView<ConnectionPropertiesDTO> listView) {
        this.listView = listView;
    }

    @Override
    protected void updateItem(ConnectionPropertiesDTO connectionDTO, boolean empty) {
        this.connectionDTO = connectionDTO;
        super.updateItem(connectionDTO, empty);

        if (empty || connectionDTO == null) {
            setText(null);
            setGraphic(null);
        } else {

            if (loader == null) {
                try {
                    loader = new FXMLLoader(SubscriptionViewCell.class.getResource("connectionButtonView.fxml"),
                            ResourceBundle.getBundle("org.correomqtt.i18n", SettingsProvider.getInstance().getSettings().getCurrentLocale()));
                    loader.setController(this);
                    loader.load();

                } catch (Exception e) {
                    LOGGER.error("Exception rendering connection:", e);
                    setText(resources.getString("commonRowCreationError"));
                    setGraphic(null);
                    return;
                }

            }
            mainNode.getStyleClass().add(SettingsProvider.getInstance().getIconModeCssClass());
            if(listView != null) {
                mainNode.prefWidthProperty().bind(listView.widthProperty().subtract(20));
            }
            setConnection(connectionDTO);
            setText(null);
            setGraphic(mainNode);
        }
    }

    private void setConnection(ConnectionPropertiesDTO connectionDTO) {

        //TODO parent css class only
        if (connectionDTO.isDirty()) {
            nameLabel.setText(connectionDTO.getName() + " *");
            nameLabel.getStyleClass().removeAll("dirty");
            nameLabel.getStyleClass().add("dirty");

        } else {
            nameLabel.setText(connectionDTO.getName());
            nameLabel.getStyleClass().removeAll("dirty");

        }

        if (ConnectionHolder.getInstance().isConnectionUnused(ConnectionTransformer.propsToDto(connectionDTO))) {
            setGray();
        } else {
            CorreoMqttClient client = ConnectionHolder.getInstance().getConnection(getConnectionId()).getClient();
            if (client == null) {
                setGray();
            } else {
                switch (client.getState()) {
                    case CONNECTED:
                        setGreen();
                        break;
                    case RECONNECTING:
                    case CONNECTING:
                        setYellow();
                        break;
                    case FAILED:
                        setRed();
                        break;
                    case DISCONNECTED:
                    case UNKOWN:
                        setGray();
                        break;
                }
            }
        }

    }

    private void setGreen() {
        Platform.runLater(() -> {
            statusLabel.getStyleClass().remove("redStatus");
            statusLabel.getStyleClass().remove("grayStatus");
            statusLabel.getStyleClass().remove("yellowStatus");
            statusLabel.getStyleClass().add("greenStatus");
        });
    }

    private void setRed() {
        Platform.runLater(() -> {
            statusLabel.getStyleClass().remove("grayStatus");
            statusLabel.getStyleClass().remove("greenStatus");
            statusLabel.getStyleClass().remove("yellowStatus");
            statusLabel.getStyleClass().add("redStatus");
        });
    }

    private void setGray() {
        Platform.runLater(() -> {
            statusLabel.getStyleClass().remove("redStatus");
            statusLabel.getStyleClass().remove("greenStatus");
            statusLabel.getStyleClass().remove("yellowStatus");
            statusLabel.getStyleClass().add("grayStatus");
        });
    }

    private void setYellow() {
        Platform.runLater(() -> {
            statusLabel.getStyleClass().remove("redStatus");
            statusLabel.getStyleClass().remove("greenStatus");
            statusLabel.getStyleClass().remove("grayStatus");
            statusLabel.getStyleClass().add("yellowStatus");
        });
    }

    @Override
    public void onDisconnectFromConnectionDeleted(String connectionId) {
        if (connectionId.equals(getConnectionId())) {
            setGray();
        }
    }

    @Override
    public void onConnect() {
        setGreen();
    }

    @Override
    public void onConnectRunning() {
        setYellow();
    }

    @Override
    public void onConnectionFailed(Throwable message) {
        setRed();
    }

    @Override
    public void onConnectionLost() {
        setRed();
    }

    @Override
    public void onDisconnect() {
        setGray();
    }

    @Override
    public void onDisconnectFailed(Throwable exception) {
        setRed();
    }

    @Override
    public void onDisconnectRunning() {
        setYellow();
    }

    @Override
    public void onConnectionReconnected() {
        setGreen();
    }

    @Override
    public void onReconnectFailed(AtomicInteger triedReconnects, int maxReconnects) {
        setRed();
    }

    @Override
    public String getConnectionId() {
        if (connectionDTO != null) {
            return connectionDTO.getId();
        }
        return "";
    }
}
