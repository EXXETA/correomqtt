package org.correomqtt.gui.views.scripting;

import javafx.fxml.FXML;
import javafx.fxml.FXMLLoader;
import javafx.scene.control.ListCell;
import javafx.scene.control.ListView;
import javafx.scene.layout.Pane;
import org.correomqtt.business.connection.ConnectionState;
import org.correomqtt.business.fileprovider.SettingsProvider;
import org.correomqtt.business.mqtt.CorreoMqttClient;
import org.correomqtt.business.utils.ConnectionHolder;
import org.correomqtt.gui.controls.IconLabel;
import org.correomqtt.gui.model.ConnectionPropertiesDTO;
import org.correomqtt.gui.model.GuiConnectionState;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.ResourceBundle;

public class ConnectionCellButton extends ListCell<ConnectionPropertiesDTO> {

    private static final Logger LOGGER = LoggerFactory.getLogger(ConnectionCellButton.class);
    private final ListView<ConnectionPropertiesDTO> listView;

    @FXML
    private Pane mainNode;
    @FXML
    private IconLabel nameLabel;
    @FXML
    private ResourceBundle resources;

    private FXMLLoader loader;

    private ConnectionPropertiesDTO connectionDTO;

    @FXML
    public void initialize() {

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
                    loader = new FXMLLoader(ConnectionCellButton.class.getResource("connectionButtonView.fxml"),
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
            if (listView != null) {
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


        CorreoMqttClient client = ConnectionHolder.getInstance().getClient(connectionDTO.getId());

        ConnectionState state = ConnectionState.DISCONNECTED_GRACEFUL;
        if (client != null) {
            state = client.getState();
        }
        GuiConnectionState guiState = GuiConnectionState.of(state);
        nameLabel.setIconColor(guiState.getIconColor());


    }

    public String getConnectionId() {
        if (connectionDTO != null) {
            return connectionDTO.getId();
        }
        return "";
    }
}
