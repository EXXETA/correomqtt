package org.correomqtt.gui.cell;

import javafx.beans.value.ChangeListener;
import javafx.beans.value.ObservableValue;
import javafx.fxml.FXML;
import javafx.fxml.FXMLLoader;
import javafx.scene.control.*;
import javafx.scene.layout.Pane;
import org.correomqtt.business.model.CorreoMqttVersion;
import org.correomqtt.business.model.Lwt;
import org.correomqtt.business.model.Proxy;
import org.correomqtt.business.model.TlsSsl;
import org.correomqtt.business.provider.SettingsProvider;
import org.correomqtt.gui.model.ConnectionPropertiesDTO;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.ResourceBundle;

public class ExportConnectionCell extends ListCell<ConnectionPropertiesDTO> {
    private static final Logger LOGGER = LoggerFactory.getLogger(ConnectionCell.class);
    public static final String DIRTY_CLASS = "dirty";
    public static final String INACTIVE_CLASS = "inactive";
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

    @FXML
    private CheckBox exportCheckbox;

    private FXMLLoader loader;

    @FXML
    public void initialize() {
        mainNode.getStyleClass().add(SettingsProvider.getInstance().getIconModeCssClass());

    }

    public ExportConnectionCell(ListView<ConnectionPropertiesDTO> listView) {
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
        nameLabel.setText(connectionDTO.getName() + " *");
        nameLabel.getStyleClass().removeAll(DIRTY_CLASS);
        exportCheckbox.setVisible(true);
        descriptionLabel.getStyleClass().removeAll(DIRTY_CLASS);

        mqtt3Tag.setVisible(false);
        mqtt5Tag.setVisible(false);
        credentialsTag.setVisible(false);
        sslTag.setVisible(false);
        proxyTag.setVisible(false);
        lwtTag.setVisible(false);

        descriptionLabel.setText(connectionDTO.getHostAndPort());

        boolean credentials = connectionDTO.getUsername() != null && !connectionDTO.getUsername().isEmpty()
                && connectionDTO.getPassword() != null && !connectionDTO.getPassword().isEmpty();
        credentialsTag.setVisible(credentials);
        credentialsTag.setManaged(credentials);

    }


}
