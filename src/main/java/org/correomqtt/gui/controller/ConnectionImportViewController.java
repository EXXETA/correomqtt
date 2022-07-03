package org.correomqtt.gui.controller;

import javafx.collections.FXCollections;
import javafx.collections.ObservableList;
import javafx.event.ActionEvent;
import javafx.fxml.FXML;
import javafx.scene.control.Button;
import javafx.scene.control.cell.CheckBoxListCell;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.layout.AnchorPane;
import javafx.stage.Stage;
import org.controlsfx.control.CheckListView;
import org.correomqtt.business.dispatcher.ImportConnectionDispatcher;
import org.correomqtt.business.dispatcher.ImportConnectionObserver;
import org.correomqtt.business.dispatcher.LogObserver;
import org.correomqtt.business.model.ConnectionConfigDTO;
import org.correomqtt.business.provider.SettingsProvider;
import org.correomqtt.business.utils.ConnectionHolder;
import org.correomqtt.gui.keyring.KeyringHandler;
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

public class ConnectionImportViewController extends BaseController implements LogObserver, ImportConnectionObserver {
    private final ConnectionImportViewDelegate delegate;
    private static final Logger LOGGER = LoggerFactory.getLogger(ConnectionExportViewController.class);
    private ObservableList<ConnectionConfigDTO> connectionConfigDTOS = FXCollections.observableArrayList();
    private ConnectionPropertiesDTO activeConnectionConfigDTO;
    private static ResourceBundle resources;

    @FXML
    private CheckListView<ConnectionConfigDTO> connectionsListView;
    @FXML
    private Button importButton;
    @FXML
    private AnchorPane containerAnchorPane;


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

        showAsDialog(result, resources.getString("connectionExportViewControllerTitle"), properties, false, false, null,
                event -> result.getController().keyHandling(event));
    }


    @FXML
    public void initialize() {
        importButton.setDisable(false);
//        containerAnchorPane.getStyleClass().add(SettingsProvider.getInstance().getIconModeCssClass());
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
        Stage stage = (Stage) importButton.getScene().getWindow();
        stage.close();
    }

    @Override
    public String getConnectionId() {
        return null;
    }

    @Override
    public void onImportSucceeded(List<ConnectionConfigDTO> connectionConfigDTOList) {
        connectionConfigDTOS.addAll(connectionConfigDTOList);
        connectionsListView.setItems(connectionConfigDTOS);
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
        connectionsListView.getCheckModel().getCheckedItems().forEach(connectionConfigDTO -> {
            if (!connections.contains(connectionConfigDTO)) {
                connections.add(connectionConfigDTO);
            }
        });
        KeyringHandler.getInstance().retryWithMasterPassword(
                masterPassword -> SettingsProvider.getInstance().saveConnections(connections, masterPassword),
                resources.getString("onPasswordSaveFailedTitle"),
                resources.getString("onPasswordSaveFailedHeader"),
                resources.getString("onPasswordSaveFailedContent"),
                resources.getString("onPasswordSaveFailedGiveUp"),
                resources.getString("onPasswordSaveFailedTryAgain")
        );
        ImportConnectionDispatcher.getInstance().onImportSucceeded(connections);
        closeDialog();


    }
}
