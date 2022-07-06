package org.correomqtt.gui.controller;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.ObjectMapper;
import javafx.collections.FXCollections;
import javafx.collections.ObservableList;
import javafx.fxml.FXML;
import javafx.scene.control.Button;
import javafx.scene.control.CheckBox;
import javafx.scene.control.PasswordField;
import javafx.scene.control.Tooltip;
import javafx.scene.control.cell.CheckBoxListCell;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.layout.AnchorPane;
import javafx.stage.FileChooser;
import javafx.stage.Stage;
import org.controlsfx.control.CheckListView;
import org.correomqtt.business.dispatcher.ExportConnectionDispatcher;
import org.correomqtt.business.dispatcher.ExportConnectionObserver;
import org.correomqtt.business.encryption.EncryptorAesGcm;
import org.correomqtt.business.model.ConnectionConfigDTO;
import org.correomqtt.business.model.ConnectionExportDTO;
import org.correomqtt.business.model.ExportConnectionView;
import org.correomqtt.business.provider.EncryptionRecoverableException;
import org.correomqtt.business.provider.SettingsProvider;
import org.correomqtt.business.utils.ConnectionHolder;
import org.correomqtt.gui.business.TaskFactory;
import org.correomqtt.gui.model.ConnectionPropertiesDTO;
import org.correomqtt.gui.model.WindowProperty;
import org.correomqtt.gui.model.WindowType;
import org.correomqtt.gui.utils.WindowHelper;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.File;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.ResourceBundle;

import static org.correomqtt.gui.controller.ConnectionSettingsViewController.EXCLAMATION_CIRCLE_SOLID;


public class ConnectionExportViewController extends BaseController implements ExportConnectionObserver {

    private final ConnectionExportViewDelegate delegate;
    private static final Logger LOGGER = LoggerFactory.getLogger(ConnectionExportViewController.class);
    private ObservableList<ConnectionConfigDTO> connectionConfigDTOS = FXCollections.observableArrayList();
    private ConnectionPropertiesDTO activeConnectionConfigDTO;

    private static ResourceBundle resources;


    @FXML
    private CheckListView<ConnectionConfigDTO> connectionsListView;
    @FXML
    private Button exportButton;
    @FXML
    private AnchorPane containerAnchorPane;
    @FXML
    private CheckBox passwordCheckBox;
    @FXML
    private PasswordField passwordField;


    public ConnectionExportViewController(ConnectionExportViewDelegate delegate) {
        this.delegate = delegate;
        ExportConnectionDispatcher.getInstance().addObserver(this);

    }

    public static LoaderResult<ConnectionExportViewController> load(ConnectionExportViewDelegate delegate) {
        return load(ConnectionExportViewController.class, "connectionExportView.fxml",
                () -> new ConnectionExportViewController(delegate));
    }

    public static void showAsDialog(ConnectionExportViewDelegate delegate) {

        LOGGER.info("OPEN DIALOG");
        Map<Object, Object> properties = new HashMap<>();
        properties.put(WindowProperty.WINDOW_TYPE, WindowType.CONNECTION_EXPORT);

        if (WindowHelper.focusWindowIfAlreadyThere(properties)) {
            return;
        }
        LoaderResult<ConnectionExportViewController> result = load(delegate);
        resources = result.getResourceBundle();

        showAsDialog(result, resources.getString("connectionExportViewControllerTitle"), properties, false, false, null,
                event -> result.getController().keyHandling(event));
    }


    @FXML
    public void initialize() {
        exportButton.setDisable(false);
        passwordCheckBox.setSelected(false);
        passwordField.setDisable(true);
        passwordField.setVisible(false);
        passwordCheckBox.selectedProperty().addListener((observableValue, oldValue, newValue) -> {
            if (newValue) {
                passwordField.setVisible(true);
                passwordField.setDisable(false);
            } else {
                passwordField.setVisible(false);
                passwordField.setDisable(true);
            }
        });
        containerAnchorPane.getStyleClass().add(SettingsProvider.getInstance().getIconModeCssClass());
        loadConnectionListFromBackground();
        connectionsListView.setCellFactory(lv -> new CheckBoxListCell<>(connectionsListView::getItemBooleanProperty) {
            @Override
            public void updateItem(ConnectionConfigDTO connectionConfigDTO, boolean empty) {
                super.updateItem(connectionConfigDTO, empty);
                setText(connectionConfigDTO == null ? "" : connectionConfigDTO.getName());
            }
        });

    }

    private void keyHandling(KeyEvent event) {
        if (KeyCode.ESCAPE == event.getCode()) {
            closeDialog();
        }
    }

    private void closeDialog() {
        Stage stage = (Stage) exportButton.getScene().getWindow();
        stage.close();
    }


    private void loadConnectionListFromBackground() {

        List<ConnectionConfigDTO> connectionList = ConnectionHolder.getInstance().getSortedConnections();
        connectionConfigDTOS.addAll(connectionList);
        connectionsListView.setItems(connectionConfigDTOS);
        LOGGER.debug("Loading connection list from background");
    }


    public void onExportClicked() {
        Stage stage = (Stage) containerAnchorPane.getScene().getWindow();

        if (passwordCheckBox.isSelected() && passwordField.getText().isEmpty()) {
            passwordField.setTooltip(new Tooltip(resources.getString("passwordEmpty")));
            passwordField.getStyleClass().add(EXCLAMATION_CIRCLE_SOLID);
            return;
        }
        FileChooser fileChooser = new FileChooser();
        fileChooser.setTitle(resources.getString("exportUtilsTitle"));
        FileChooser.ExtensionFilter extFilter = new FileChooser.ExtensionFilter(resources.getString("exportUtilsDescription"), "*.json");
        fileChooser.getExtensionFilters().add(extFilter);

        File file = fileChooser.showSaveDialog(stage);
        if (file != null) {
            if (passwordCheckBox.isSelected()) {
                try {
                    String connectionsJSON = new ObjectMapper().writerWithView(ExportConnectionView.class).writeValueAsString(connectionsListView.getCheckModel().getCheckedItems());
                    String encryptedData = new EncryptorAesGcm(passwordField.getText()).encrypt(connectionsJSON);
                    ConnectionExportDTO connectionExportDTO = new ConnectionExportDTO(EncryptorAesGcm.ENCRYPTION_TRANSFORMATION, encryptedData);
                    TaskFactory.exportConnection(null, file, connectionExportDTO);

                } catch (JsonProcessingException | EncryptionRecoverableException e) {
                    ExportConnectionDispatcher.getInstance().onExportFailed(file, e);
                }

            } else {
//                    String connectionsJSON = new ObjectMapper().writerWithView(ExportConnectionView.class).writeValueAsString(connectionsListView.getCheckModel().getCheckedItems());
                    List<ConnectionConfigDTO>  connectionConfigDTOS = connectionsListView.getCheckModel().getCheckedItems();
                    ConnectionExportDTO connectionExportDTO = new ConnectionExportDTO(connectionConfigDTOS);
                    TaskFactory.exportConnection(null, file, connectionExportDTO);


            }

        }
    }

    @FXML
    public void onCancelClicked() {
        Stage stage = (Stage) exportButton.getScene().getWindow();
        stage.close();
    }

    @Override
    public void onExportSucceeded() {
        closeDialog();
    }
}
