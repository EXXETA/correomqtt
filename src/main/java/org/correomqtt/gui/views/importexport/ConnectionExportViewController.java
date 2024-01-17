package org.correomqtt.gui.views.importexport;

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
import org.correomqtt.business.concurrent.TaskErrorResult;
import org.correomqtt.business.fileprovider.SettingsProvider;
import org.correomqtt.business.importexport.connections.ExportConnectionsTask;
import org.correomqtt.business.model.ConnectionConfigDTO;
import org.correomqtt.business.utils.ConnectionHolder;
import org.correomqtt.gui.model.ConnectionPropertiesDTO;
import org.correomqtt.gui.model.WindowProperty;
import org.correomqtt.gui.model.WindowType;
import org.correomqtt.gui.transformer.ConnectionTransformer;
import org.correomqtt.gui.utils.AlertHelper;
import org.correomqtt.gui.utils.WindowHelper;
import org.correomqtt.gui.views.LoaderResult;
import org.correomqtt.gui.views.base.BaseControllerImpl;
import org.correomqtt.gui.views.connectionsettings.ConnectionSettingsViewController;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.File;
import java.text.MessageFormat;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.ResourceBundle;

public class ConnectionExportViewController extends BaseControllerImpl {

    private static final Logger LOGGER = LoggerFactory.getLogger(ConnectionExportViewController.class);
    @FXML
    private Label passwordLabel;

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
    private void initialize() {

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
        return cell;
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
        connectionsListView.setItems(FXCollections.observableArrayList(ConnectionTransformer.dtoListToPropList(connectionList)));
        LOGGER.debug("Loading connection list from background");
    }


    public void onExportClicked() {

        ObservableList<ConnectionPropertiesDTO> checkedItems = connectionsListView.getCheckModel().getCheckedItems();
        Stage stage = (Stage) containerAnchorPane.getScene().getWindow();
        FileChooser fileChooser = new FileChooser();
        fileChooser.setTitle(resources.getString("exportUtilsTitle"));
        fileChooser.setInitialFileName(".cqc");
        FileChooser.ExtensionFilter extFilter = new FileChooser.ExtensionFilter(resources.getString("exportUtilsDescription"), "*.cqc");
        fileChooser.getExtensionFilters().add(extFilter);
        File file = fileChooser.showSaveDialog(stage);

        new ExportConnectionsTask(file,
                ConnectionTransformer.propsListToDtoList(checkedItems),
                passwordCheckBox.isSelected() ? passwordField.getText() : null)
                .onSuccess(this::onExportSucceeded)
                .onError(this::onExportFailed)
                .run();
    }

    private void onExportSucceeded(Integer exportedSize) {
        AlertHelper.info(resources.getString("exportConnectionsSuccessTitle"),
                MessageFormat.format(resources.getString("exportConnectionsSuccessBody"),
                        exportedSize),
                true);
        this.closeDialog();
    }

    private void onExportFailed(TaskErrorResult<ExportConnectionsTask.Error> errorResult) {
        if (errorResult.isExpected()) {
            switch (errorResult.getExpectedError()) {
                case EMPTY_COLLECTION_LIST:
                    AlertHelper.warn(resources.getString("exportConnectionsEmptyTitle"),
                            resources.getString("exportConnectionsEmptyBody"),
                            true);
                    break;
                case EMPTY_PASSWORD:
                    passwordField.setTooltip(new Tooltip(resources.getString("passwordEmpty")));
                    passwordField.getStyleClass().add(ConnectionSettingsViewController.EXCLAMATION_CIRCLE_SOLID);
                    break;
                case FILE_IS_NULL:
                    // nothing to do -> file dialog was cancelled
                    break;
                case MISSING_FILE_EXTENSION:
                    AlertHelper.warn(resources.getString("exportConnectionsFilenameTitle"),
                            resources.getString("exportConnectionsFilenameBody"),
                            true);
                    break;
            }
        } else {
            AlertHelper.unexpectedAlert(errorResult.getUnexpectedError());
        }
    }

    @FXML
    private void onCancelClicked() {
        closeDialog();
    }


    public void checkAll() {
        connectionsListView.getCheckModel().checkAll();
    }

    public void checkNone() {
        connectionsListView.getCheckModel().clearChecks();
    }
}
