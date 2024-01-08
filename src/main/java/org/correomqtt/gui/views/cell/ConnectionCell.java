package org.correomqtt.gui.views.cell;

import javafx.fxml.FXML;
import javafx.fxml.FXMLLoader;
import javafx.scene.control.Label;
import javafx.scene.control.ListCell;
import javafx.scene.control.ListView;
import javafx.scene.layout.Pane;
import org.correomqtt.business.connection.ConnectionState;
import org.correomqtt.business.model.CorreoMqttVersion;
import org.correomqtt.business.model.Lwt;
import org.correomqtt.business.model.Proxy;
import org.correomqtt.business.model.TlsSsl;
import org.correomqtt.business.fileprovider.SettingsProvider;
import org.correomqtt.business.mqtt.CorreoMqttClient;
import org.correomqtt.business.utils.ConnectionHolder;
import org.correomqtt.gui.controls.IconLabel;
import org.correomqtt.gui.controls.ThemedFontIcon;
import org.correomqtt.gui.model.ConnectionPropertiesDTO;
import org.correomqtt.gui.model.GuiConnectionState;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.ResourceBundle;

@SuppressWarnings("java:S110")
public class ConnectionCell extends ListCell<ConnectionPropertiesDTO> {

    private static final Logger LOGGER = LoggerFactory.getLogger(ConnectionCell.class);
    public static final String DIRTY_CLASS = "dirty";
    public static final String INACTIVE_CLASS = "inactive";
    private final ListView<ConnectionPropertiesDTO> listView;

    @FXML
    private Pane mainNode;
    @FXML
    private IconLabel nameLabel;
    @FXML
    private Label descriptionLabel;
    @FXML
    private Label credentialsTag;
    @FXML
    private Label sslTag;
    @FXML
    private Label proxyTag;
    @FXML
    private Label lwtTag;
    @FXML
    private Label mqtt3Tag;
    @FXML
    private Label mqtt5Tag;
    @FXML
    private ResourceBundle resources;
    private FXMLLoader loader;

    @FXML
    public void initialize() {
        mainNode.getStyleClass().add(SettingsProvider.getInstance().getIconModeCssClass());
    }

    public ConnectionCell(ListView<ConnectionPropertiesDTO> listView) {
        this.listView = listView;
    }

    @Override
    protected void updateItem(ConnectionPropertiesDTO connectionDTO, boolean empty) {
        super.updateItem(connectionDTO, empty);

        if (empty || connectionDTO == null) {
            setText(null);
            setGraphic(null);
        } else {

            if (loader == null) {
                try {
                    loader = new FXMLLoader(ConnectionCell.class.getResource("connectionCell.fxml"),
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
            mainNode.prefWidthProperty().bind(listView.widthProperty().subtract(20));
            setConnection(connectionDTO);
            setText(null);
            setGraphic(mainNode);
        }
    }

    private void setConnection(ConnectionPropertiesDTO connectionDTO) {

        //TODO parent css class only
        if (connectionDTO.isDirty() || connectionDTO.isNew()) {
            nameLabel.setText(connectionDTO.getName() + " *");
            nameLabel.getStyleClass().removeAll(DIRTY_CLASS);
            nameLabel.getStyleClass().add(DIRTY_CLASS);

            descriptionLabel.getStyleClass().removeAll(DIRTY_CLASS);
            descriptionLabel.getStyleClass().add(DIRTY_CLASS);

            mqtt3Tag.getStyleClass().removeAll(INACTIVE_CLASS);
            mqtt3Tag.getStyleClass().add(INACTIVE_CLASS);

            mqtt5Tag.getStyleClass().removeAll(INACTIVE_CLASS);
            mqtt5Tag.getStyleClass().add(INACTIVE_CLASS);

            credentialsTag.getStyleClass().removeAll(INACTIVE_CLASS);
            credentialsTag.getStyleClass().add(INACTIVE_CLASS);

            sslTag.getStyleClass().removeAll(INACTIVE_CLASS);
            sslTag.getStyleClass().add(INACTIVE_CLASS);

            proxyTag.getStyleClass().removeAll(INACTIVE_CLASS);
            proxyTag.getStyleClass().add(INACTIVE_CLASS);

            lwtTag.getStyleClass().removeAll(INACTIVE_CLASS);
            lwtTag.getStyleClass().add(INACTIVE_CLASS);


        } else {
            nameLabel.setText(connectionDTO.getName());
            nameLabel.getStyleClass().removeAll(DIRTY_CLASS);
            descriptionLabel.getStyleClass().removeAll(DIRTY_CLASS);
            mqtt3Tag.getStyleClass().removeAll(INACTIVE_CLASS);
            mqtt5Tag.getStyleClass().removeAll(INACTIVE_CLASS);
            credentialsTag.getStyleClass().removeAll(INACTIVE_CLASS);
            sslTag.getStyleClass().removeAll(INACTIVE_CLASS);
            proxyTag.getStyleClass().removeAll(INACTIVE_CLASS);
            lwtTag.getStyleClass().removeAll(INACTIVE_CLASS);
        }

        if (connectionDTO.isNew()) {
            descriptionLabel.setText(null);

            mqtt3Tag.setVisible(false);
            mqtt3Tag.setManaged(false);

            mqtt5Tag.setVisible(false);
            mqtt5Tag.setManaged(false);
        } else {
            descriptionLabel.setText(connectionDTO.getHostAndPort());

            boolean mqtt3 = connectionDTO.getMqttVersionProperty().getValue().equals(CorreoMqttVersion.MQTT_3_1_1);
            mqtt3Tag.setVisible(mqtt3);
            mqtt3Tag.setManaged(mqtt3);

            boolean mqtt5 = connectionDTO.getMqttVersionProperty().getValue().equals(CorreoMqttVersion.MQTT_5_0);
            mqtt5Tag.setVisible(mqtt5);
            mqtt5Tag.setManaged(mqtt5);
        }

        boolean credentials = connectionDTO.getUsername() != null && !connectionDTO.getUsername().isEmpty()
                && connectionDTO.getPassword() != null && !connectionDTO.getPassword().isEmpty();
        credentialsTag.setVisible(credentials);
        credentialsTag.setManaged(credentials);

        boolean ssl = connectionDTO.getSslProperty().getValue().equals(TlsSsl.KEYSTORE);
        sslTag.setVisible(ssl);
        sslTag.setManaged(ssl);

        boolean proxy = connectionDTO.getProxyProperty().getValue().equals(Proxy.SSH);
        proxyTag.setVisible(proxy);
        proxyTag.setManaged(proxy);

        boolean lwt = connectionDTO.getLwtProperty().getValue().equals(Lwt.ON);
        lwtTag.setVisible(lwt);
        lwtTag.setManaged(lwt);

        CorreoMqttClient client = ConnectionHolder.getInstance().getClient(connectionDTO.getId());

        ConnectionState state = ConnectionState.DISCONNECTED_GRACEFUL;
        if (client != null) {
            state = client.getState();
        }
        GuiConnectionState guiState = GuiConnectionState.of(state);
        nameLabel.setIconColor(guiState.getIconColor());
    }


}
