package com.exxeta.correomqtt.gui.cell;

import org.correomqtt.business.model.CorreoMqttVersion;
import org.correomqtt.business.model.Lwt;
import org.correomqtt.business.model.Proxy;
import org.correomqtt.business.model.TlsSsl;
import com.exxeta.correomqtt.business.services.ConfigService;
import com.exxeta.correomqtt.gui.model.ConnectionPropertiesDTO;
import javafx.fxml.FXML;
import javafx.fxml.FXMLLoader;
import javafx.scene.control.Label;
import javafx.scene.control.ListCell;
import javafx.scene.control.ListView;
import javafx.scene.layout.Pane;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.ResourceBundle;

public class ConnectionCell extends ListCell<ConnectionPropertiesDTO> {

    private static final Logger LOGGER = LoggerFactory.getLogger(ConnectionCell.class);
    private final ListView<ConnectionPropertiesDTO> listView;

    @FXML
    private Pane mainNode;
    @FXML
    private Label nameLabel;
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
        mainNode.getStyleClass().add(ConfigService.getInstance().getThemeSettings().getActiveTheme().getIconMode());
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
                    loader = new FXMLLoader(SubscriptionViewCell.class.getResource("connectionView.fxml"),
                            ResourceBundle.getBundle("com.exxeta.correomqtt.i18n", ConfigService.getInstance().getSettings().getCurrentLocale()));
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
        if (connectionDTO.isDirty()) {
            nameLabel.setText(connectionDTO.getName() + " *");
            nameLabel.getStyleClass().removeAll("dirty");
            nameLabel.getStyleClass().add("dirty");

            descriptionLabel.getStyleClass().removeAll("dirty");
            descriptionLabel.getStyleClass().add("dirty");

            mqtt3Tag.getStyleClass().removeAll("inactive");
            mqtt3Tag.getStyleClass().add("inactive");

            mqtt5Tag.getStyleClass().removeAll("inactive");
            mqtt5Tag.getStyleClass().add("inactive");

            credentialsTag.getStyleClass().removeAll("inactive");
            credentialsTag.getStyleClass().add("inactive");

            sslTag.getStyleClass().removeAll("inactive");
            sslTag.getStyleClass().add("inactive");

            proxyTag.getStyleClass().removeAll("inactive");
            proxyTag.getStyleClass().add("inactive");

            lwtTag.getStyleClass().removeAll("inactive");
            lwtTag.getStyleClass().add("inactive");
        } else {
            nameLabel.setText(connectionDTO.getName());
            nameLabel.getStyleClass().removeAll("dirty");
            descriptionLabel.getStyleClass().removeAll("dirty");
            mqtt3Tag.getStyleClass().removeAll("inactive");
            mqtt5Tag.getStyleClass().removeAll("inactive");
            credentialsTag.getStyleClass().removeAll("inactive");
            sslTag.getStyleClass().removeAll("inactive");
            proxyTag.getStyleClass().removeAll("inactive");
            lwtTag.getStyleClass().removeAll("inactive");
        }

        descriptionLabel.setText(connectionDTO.getHostAndPort());

        boolean mqtt3 = connectionDTO.getMqttVersionProperty().getValue().equals(CorreoMqttVersion.MQTT_3_1_1);
        mqtt3Tag.setVisible(mqtt3);
        mqtt3Tag.setManaged(mqtt3);

        boolean mqtt5 = connectionDTO.getMqttVersionProperty().getValue().equals(CorreoMqttVersion.MQTT_5_0);
        mqtt5Tag.setVisible(mqtt5);
        mqtt5Tag.setManaged(mqtt5);

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
    }
}
