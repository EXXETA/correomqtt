package org.correomqtt.gui.controller;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.ObjectMapper;
import javafx.collections.FXCollections;
import javafx.collections.ObservableList;
import javafx.event.ActionEvent;
import javafx.fxml.FXML;
import javafx.scene.control.Button;
import javafx.scene.control.Label;
import javafx.scene.control.PasswordField;
import javafx.scene.control.Tooltip;
import javafx.scene.control.cell.CheckBoxListCell;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.layout.AnchorPane;
import javafx.stage.Stage;
import org.controlsfx.control.CheckListView;
import org.correomqtt.business.dispatcher.ImportConnectionDispatcher;
import org.correomqtt.business.dispatcher.ImportConnectionObserver;
import org.correomqtt.business.dispatcher.LogObserver;
import org.correomqtt.business.encryption.EncryptorAesGcm;
import org.correomqtt.business.model.ConnectionConfigDTO;
import org.correomqtt.business.model.ConnectionExportDTO;
import org.correomqtt.business.provider.EncryptionRecoverableException;
import org.correomqtt.business.provider.SettingsProvider;
import org.correomqtt.business.utils.ConnectionHolder;
import org.correomqtt.gui.keyring.KeyringHandler;
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

public class ConnectionImportViewController extends BaseController implements LogObserver, ImportConnectionObserver {
    private final ConnectionImportViewDelegate delegate;
    private static final Logger LOGGER = LoggerFactory.getLogger(ConnectionExportViewController.class);
    private ObservableList<ConnectionConfigDTO> connectionConfigDTOS = FXCollections.observableArrayList();
    private static ResourceBundle resources;
    private ConnectionExportDTO connectionExportDTO;

    @FXML
    private CheckListView<ConnectionConfigDTO> connectionsListView;
    @FXML
    private Button importButton;
    @FXML
    private AnchorPane containerAnchorPane;
    @FXML
    private PasswordField passwordField;
    @FXML
    private Button decryptButton;
    @FXML
    private Label passwordRequiredLabel;
    @FXML
    private Label passwordIncorrectLabel;
    @FXML
    private Label importConnectionsExists;


    public ConnectionImportViewController(ConnectionImportViewDelegate delegate) {
        this.delegate = delegate;
        ImportConnectionDispatcher.getInstance().addObserver(this);

    }

    public static LoaderResult<ConnectionImportViewController> load(ConnectionImportViewDelegate delegate) {
        return load(ConnectionImportViewController.class, "connectionImportView.fxml",
                () -> new ConnectionImportViewController(delegate));
    }


    public static void showAsDialog(ConnectionImportViewDelegate delegate) {

        LOGGER.info("OPEN DIALOG");
        Map<Object, Object> properties = new HashMap<>();
        properties.put(WindowProperty.WINDOW_TYPE, WindowType.CONNECTION_EXPORT);

        if (WindowHelper.focusWindowIfAlreadyThere(properties)) {
            return;
        }
        LoaderResult<ConnectionImportViewController> result = load(delegate);
        resources = result.getResourceBundle();

        showAsDialog(result, resources.getString("connectionImportViewControllerTitle"), properties, false, false, null,
                event -> result.getController().keyHandling(event));
    }


    @FXML
    public void initialize() {
        importButton.setDisable(false);
        decryptButton.setVisible(false);
        passwordField.setVisible(false);
        passwordRequiredLabel.setVisible(false);
        passwordIncorrectLabel.setVisible(false);
        importConnectionsExists.setVisible(false);
        setUpCells(null);

    }

    private void setUpCells(List<ConnectionConfigDTO> existingConnections) {
        connectionsListView.setCellFactory(lv -> new CheckBoxListCell<>(connectionsListView::getItemBooleanProperty) {
            @Override
            public void updateItem(ConnectionConfigDTO newConnection, boolean empty) {
                super.updateItem(newConnection, empty);
                setText(newConnection == null ? "" : newConnection.getName());
                setStyle("-fx-pref-height: 39;" +
                        "-fx-padding: 10");
                if (existingConnections != null) {

                    if (existingConnections.stream().anyMatch(existingConnection -> newConnection != null && (
                            existingConnection.getId().equals(newConnection.getId())
                                    || existingConnection.getName().equals(newConnection.getName())))) {
                        setDisable(true);
                        importConnectionsExists.setVisible(true);
                    }

                } else setDisable(false);

            }
        });
    }

    private void keyHandling(KeyEvent event) {
        if (KeyCode.ESCAPE == event.getCode()) {
            closeDialog();
        }
    }

    private void closeDialog() {
        Stage stage = (Stage) importButton.getScene().getWindow();
        stage.close();
    }

    @Override
    public String getConnectionId() {
        return null;
    }

    @Override
    public void onImportSucceeded(ConnectionExportDTO connectionExportDTO) {
        List<ConnectionConfigDTO> importedConnections;
        if (connectionExportDTO != null) {

            if (connectionExportDTO.getEncryptionType() != null) {
                passwordField.setVisible(true);
                decryptButton.setVisible(true);
                importButton.setDisable(true);
                passwordRequiredLabel.setVisible(true);
                this.connectionExportDTO = connectionExportDTO;
            } else {
                importedConnections = connectionExportDTO.getConnectionConfigDTOS();
                connectionConfigDTOS.addAll(importedConnections);
                connectionsListView.setItems(connectionConfigDTOS);
                setUpCells(ConnectionHolder.getInstance().getSortedConnections());

            }
        }

    }


    @Override
    public void onImportCancelled(File file) {
        closeDialog();
    }

    @Override
    public void onImportFailed(File file, Throwable exception) {
        closeDialog();
    }


    @Override
    public void updateLog(String message) {

    }

    public void onImportClicked() {
        importConnections();
    }

    public void onCancelClicked(ActionEvent actionEvent) {
        closeDialog();
    }

    public void importConnections() {
        List<ConnectionConfigDTO> connections = ConnectionHolder.getInstance().getSortedConnections();
        connections.addAll(connectionsListView.getCheckModel().getCheckedItems());
        KeyringHandler.getInstance().retryWithMasterPassword(
                masterPassword -> SettingsProvider.getInstance().saveConnections(connections, masterPassword),
                resources.getString("onPasswordSaveFailedTitle"),
                resources.getString("onPasswordSaveFailedHeader"),
                resources.getString("onPasswordSaveFailedContent"),
                resources.getString("onPasswordSaveFailedGiveUp"),
                resources.getString("onPasswordSaveFailedTryAgain")
        );
        ImportConnectionDispatcher.getInstance().onImportSucceeded(connectionExportDTO);
        closeDialog();


    }

    public void onDecryptClicked() {
        if (passwordField.getText() != null) {
            try {
                String connectionsString = new EncryptorAesGcm(passwordField.getText()).decrypt(this.connectionExportDTO.getEncryptedData());
                List<ConnectionConfigDTO> connectionConfigDTOList = new ObjectMapper().readerFor(new TypeReference<List<ConnectionConfigDTO>>() {
                }).readValue(connectionsString);
                connectionConfigDTOS.addAll(connectionConfigDTOList);
                connectionsListView.setItems(connectionConfigDTOS);
                setUpCells(ConnectionHolder.getInstance().getSortedConnections());
                importButton.setDisable(false);
                decryptButton.setVisible(false);
                passwordField.setVisible(false);
                passwordRequiredLabel.setVisible(false);
                passwordIncorrectLabel.setVisible(false);
            } catch (JsonProcessingException e) {
                ImportConnectionDispatcher.getInstance().onImportFailed(null, e);
            } catch (EncryptionRecoverableException e) {
                passwordIncorrectLabel.setVisible(true);
            }
        } else {
            passwordField.setTooltip(new Tooltip(resources.getString("passwordEmpty")));
            passwordField.getStyleClass().add(EXCLAMATION_CIRCLE_SOLID);
        }
    }
}
