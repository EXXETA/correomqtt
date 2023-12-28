package org.correomqtt.gui.controller;

import javafx.collections.FXCollections;
import javafx.collections.ObservableList;
import javafx.fxml.FXML;
import javafx.scene.control.Button;
import javafx.scene.control.CheckBox;
import javafx.scene.control.Label;
import javafx.scene.control.ListCell;
import javafx.scene.control.ListView;
import javafx.scene.control.PasswordField;
import javafx.scene.control.Tooltip;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.layout.AnchorPane;
import javafx.stage.FileChooser;
import javafx.stage.Stage;
import org.controlsfx.control.CheckListView;
import org.correomqtt.business.dispatcher.ExportConnectionDispatcher;
import org.correomqtt.business.dispatcher.ExportConnectionObserver;
import org.correomqtt.business.model.ConnectionConfigDTO;
import org.correomqtt.business.provider.SettingsProvider;
import org.correomqtt.business.utils.ConnectionHolder;
import org.correomqtt.gui.business.ExportTaskFactory;
import org.correomqtt.gui.cell.ExportConnectionCell;
import org.correomqtt.gui.helper.AlertHelper;
import org.correomqtt.gui.model.ConnectionPropertiesDTO;
import org.correomqtt.gui.model.WindowProperty;
import org.correomqtt.gui.model.WindowType;
import org.correomqtt.gui.transformer.ConnectionTransformer;
import org.correomqtt.gui.utils.WindowHelper;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.File;
import java.text.MessageFormat;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.ResourceBundle;

import static org.correomqtt.gui.controller.ConnectionSettingsViewController.EXCLAMATION_CIRCLE_SOLID;


public class ConnectionExportViewController extends BaseControllerImpl implements ExportConnectionObserver {

    private static final Logger LOGGER = LoggerFactory.getLogger(ConnectionExportViewController.class);
    @FXML
    public Label passwordLabel;

    private static ResourceBundle resources;

    @FXML
    private CheckListView<ConnectionPropertiesDTO> connectionsListView;
    @FXML
    private Button exportButton;
    @FXML
    private AnchorPane containerAnchorPane;
    @FXML
    private CheckBox passwordCheckBox;
    @FXML
    private PasswordField passwordField;


    public ConnectionExportViewController() {
        ExportConnectionDispatcher.getInstance().addObserver(this);
    }

    public static LoaderResult<ConnectionExportViewController> load() {
        return load(ConnectionExportViewController.class, "connectionExportView.fxml",
                ConnectionExportViewController::new);
    }

    public static void showAsDialog() {

        LOGGER.info("OPEN DIALOG");
        Map<Object, Object> properties = new HashMap<>();
        properties.put(WindowProperty.WINDOW_TYPE, WindowType.CONNECTION_EXPORT);

        if (WindowHelper.focusWindowIfAlreadyThere(properties)) {
            return;
        }
        LoaderResult<ConnectionExportViewController> result = load();
        resources = result.getResourceBundle();

        showAsDialog(result, resources.getString("connectionExportViewControllerTitle"), properties, false, false, null,
                event -> result.getController().keyHandling(event));
    }


    @FXML
    public void initialize() {

        // improve list style

        exportButton.setDisable(false);
        passwordCheckBox.setSelected(false);
        passwordField.setDisable(true);
        passwordField.setVisible(false);
        passwordLabel.setVisible(false);
        passwordLabel.setDisable(true);
        passwordCheckBox.selectedProperty().addListener((observableValue, oldValue, newValue) -> {
            if (Boolean.TRUE.equals(newValue)) {
                passwordField.setVisible(true);
                passwordField.setDisable(false);
                passwordLabel.setVisible(true);
                passwordLabel.setDisable(false);
            } else {
                passwordField.setVisible(false);
                passwordField.setDisable(true);
                passwordLabel.setVisible(false);
                passwordLabel.setDisable(true);
            }
        });
        containerAnchorPane.getStyleClass().add(SettingsProvider.getInstance().getIconModeCssClass());
        connectionsListView.setCellFactory(this::createCell);
        loadConnectionListFromBackground();
        checkAll();
    }

    private ListCell<ConnectionPropertiesDTO> createCell(ListView<ConnectionPropertiesDTO> connectionConfigDTOListView) {

        ExportConnectionCell cell = new ExportConnectionCell(connectionsListView);
        cell.selectedProperty().addListener((observable, oldValue, newValue) ->
                connectionsListView.getSelectionModel().clearSelection());


        cell.setOnMouseMoved(e -> {

        });
        return cell;
    }

    private void keyHandling(KeyEvent event) {
        if (KeyCode.ESCAPE == event.getCode()) {
            closeDialog();
        }
    }

    private void closeDialog() {
        cleanUp();
        Stage stage = (Stage) exportButton.getScene().getWindow();
        stage.close();
    }

    private void cleanUp() {
        ExportConnectionDispatcher.getInstance().removeObserver(this);
    }

    private void loadConnectionListFromBackground() {

        List<ConnectionConfigDTO> connectionList = ConnectionHolder.getInstance().getSortedConnections();
        connectionsListView.setItems(FXCollections.observableArrayList(ConnectionTransformer.dtoListToPropList(connectionList)));
        LOGGER.debug("Loading connection list from background");
    }


    public void onExportClicked() {

        ObservableList<ConnectionPropertiesDTO> checkedItems = connectionsListView.getCheckModel().getCheckedItems();

        if (checkedItems.isEmpty()) {
            AlertHelper.warn(resources.getString("exportConnectionsEmptyTitle"),
                    resources.getString("exportConnectionsEmptyBody"),
                    true);
            return;
        }

        if (passwordCheckBox.isSelected() && passwordField.getText().isEmpty()) {
            passwordField.setTooltip(new Tooltip(resources.getString("passwordEmpty")));
            passwordField.getStyleClass().add(EXCLAMATION_CIRCLE_SOLID);
            return;
        }

        Stage stage = (Stage) containerAnchorPane.getScene().getWindow();
        FileChooser fileChooser = new FileChooser();
        fileChooser.setTitle(resources.getString("exportUtilsTitle"));
        fileChooser.setInitialFileName(".cqc");
        FileChooser.ExtensionFilter extFilter = new FileChooser.ExtensionFilter(resources.getString("exportUtilsDescription"), "*.cqc");
        fileChooser.getExtensionFilters().add(extFilter);
        File file = fileChooser.showSaveDialog(stage);

        if(file == null){
            // file chooser was cancelled e.g. with ESC
            return;
        }

        if (!file.getName().endsWith(".cqc")) {
            AlertHelper.warn(resources.getString("exportConnectionsFilenameTitle"),
                    resources.getString("exportConnectionsFilenameBody"),
                    true);
            return;
        }

        ExportTaskFactory.exportConnections(file,
                ConnectionTransformer.propsListToDtoList(checkedItems),
                passwordCheckBox.isSelected() ? passwordField.getText() : null);
    }

    @FXML
    public void onCancelClicked() {
        closeDialog();
    }

    @Override
    public void onExportSucceeded() {

        AlertHelper.info(resources.getString("exportConnectionsSuccessTitle"),
                MessageFormat.format(resources.getString("exportConnectionsSuccessBody"),
                        connectionsListView.getCheckModel().getCheckedItems().size()),
                true);
        closeDialog();
    }

    public void checkAll() {
        connectionsListView.getCheckModel().checkAll();
    }

    public void checkNone() {
        connectionsListView.getCheckModel().clearChecks();
    }
}
