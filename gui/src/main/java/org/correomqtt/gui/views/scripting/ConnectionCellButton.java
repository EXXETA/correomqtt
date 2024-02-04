package org.correomqtt.gui.views.scripting;

import org.correomqtt.di.Assisted;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import javafx.fxml.FXML;
import javafx.fxml.FXMLLoader;
import javafx.scene.control.ListCell;
import javafx.scene.control.ListView;
import javafx.scene.layout.Pane;
import org.correomqtt.core.connection.ConnectionState;
import org.correomqtt.core.settings.SettingsManager;
import org.correomqtt.core.mqtt.CorreoMqttClient;
import org.correomqtt.core.utils.ConnectionManager;
import org.correomqtt.gui.controls.IconLabel;
import org.correomqtt.gui.model.ConnectionPropertiesDTO;
import org.correomqtt.gui.model.GuiConnectionState;
import org.correomqtt.gui.theme.ThemeManager;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.ResourceBundle;

@DefaultBean
public class ConnectionCellButton extends ListCell<ConnectionPropertiesDTO> {

    private static final Logger LOGGER = LoggerFactory.getLogger(ConnectionCellButton.class);
    private final ConnectionManager connectionManager;
    private final SettingsManager settingsManager;
    private final ThemeManager themeManager;
    private final ListView<ConnectionPropertiesDTO> listView;

    @FXML
    private Pane mainNode;
    @FXML
    private IconLabel nameLabel;
    @FXML
    private ResourceBundle resources;

    private FXMLLoader loader;

    private ConnectionPropertiesDTO connectionDTO;


    @Inject
    public ConnectionCellButton(ConnectionManager connectionManager,
                                SettingsManager settingsManager,
                                ThemeManager themeManager,
                                @Assisted ListView<ConnectionPropertiesDTO> listView) {
        this.connectionManager = connectionManager;
        this.settingsManager = settingsManager;
        this.themeManager = themeManager;
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
                            ResourceBundle.getBundle("org.correomqtt.i18n", settingsManager.getSettings().getCurrentLocale()));
                    loader.setController(this);
                    loader.load();

                } catch (Exception e) {
                    LOGGER.error("Exception rendering connection:", e);
                    setText(resources.getString("commonRowCreationError"));
                    setGraphic(null);
                    return;
                }

            }
            mainNode.getStyleClass().add(themeManager.getIconModeCssClass());
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


        CorreoMqttClient client = connectionManager.getClient(connectionDTO.getId());

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
