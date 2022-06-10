package org.correomqtt.gui.controller;

import javafx.collections.FXCollections;
import javafx.collections.ObservableList;
import javafx.event.ActionEvent;
import javafx.fxml.FXML;
import javafx.scene.control.Button;
import javafx.scene.control.ListCell;
import javafx.scene.control.ListView;
import javafx.scene.control.SelectionMode;
import javafx.scene.input.DragEvent;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.layout.AnchorPane;
import javafx.stage.FileChooser;
import javafx.stage.Stage;
import org.correomqtt.business.model.ConnectionConfigDTO;
import org.correomqtt.business.provider.SettingsProvider;
import org.correomqtt.business.utils.ConnectionHolder;
import org.correomqtt.gui.business.TaskFactory;
import org.correomqtt.gui.cell.ConnectionCell;
import org.correomqtt.gui.cell.ExportConnectionCell;
import org.correomqtt.gui.model.ConnectionPropertiesDTO;
import org.correomqtt.gui.model.WindowProperty;
import org.correomqtt.gui.model.WindowType;
import org.correomqtt.gui.transformer.ConnectionTransformer;
import org.correomqtt.gui.utils.MessageUtils;
import org.correomqtt.gui.utils.WindowHelper;
import org.correomqtt.plugin.manager.PluginManager;
import org.correomqtt.plugin.model.LwtConnectionExtensionDTO;
import org.correomqtt.plugin.spi.LwtSettingsHook;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.File;
import java.util.*;

public class ConnectionExportViewController  extends BaseController {

    private final ConnectionExportViewDelegate delegate;
    private static final Logger LOGGER = LoggerFactory.getLogger(ConnectionExportViewController.class);


    @FXML
    private ListView<ConnectionPropertiesDTO> connectionsListView;
    @FXML
    private Button exportButton;
    @FXML
    private AnchorPane containerAnchorPane;

    private List<ConnectionConfigDTO> connectionConfigDTOS = new ArrayList<>();
    private ConnectionPropertiesDTO activeConnectionConfigDTO;

    private static ResourceBundle resources;


    public ConnectionExportViewController(ConnectionExportViewDelegate delegate) {
        this.delegate = delegate;

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
        containerAnchorPane.getStyleClass().add(SettingsProvider.getInstance().getIconModeCssClass());
        loadConnectionListFromBackground();
        connectionsListView.getSelectionModel().setSelectionMode(SelectionMode.MULTIPLE);
        connectionsListView.setCellFactory(this::createCell);


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

    private ListCell<ConnectionPropertiesDTO> createCell(ListView<ConnectionPropertiesDTO> connectionListView) {
        ExportConnectionCell cell = new ExportConnectionCell(connectionListView);
//        cell.setOnMouseClicked(mouseEvent -> {
//            connectionsListView.getSelectionModel().select();
//
//        });
        cell.selectedProperty().addListener((observable, oldValue, newValue) -> {

            List<ConnectionPropertiesDTO> selectedItem = connectionsListView.getSelectionModel().getSelectedItems();

            if (selectedItem == activeConnectionConfigDTO) {
                return;
            }


        });


        return cell;
    }

    private void loadConnectionListFromBackground() {
        ObservableList<ConnectionPropertiesDTO> list = FXCollections.observableArrayList(ConnectionPropertiesDTO.extractor());
        connectionConfigDTOS = ConnectionHolder.getInstance().getSortedConnections();
        connectionConfigDTOS.forEach(c -> list.add(ConnectionTransformer.dtoToProps(c)));
        connectionsListView.setItems(list);
        executeOnLoadSettingsExtensions();
        LOGGER.debug("Loading connection list from background");
    }

    private void executeOnLoadSettingsExtensions() {
        connectionsListView.getItems().forEach(c -> {
//            decodeLwtPayload(c);
            LwtConnectionExtensionDTO connectionExtensionDTO = new LwtConnectionExtensionDTO(c);
            for (LwtSettingsHook p : PluginManager.getInstance().getExtensions(LwtSettingsHook.class)) {
                connectionExtensionDTO = p.onLoadConnection(connectionExtensionDTO);
            }
            connectionExtensionDTO.merge(c);
        });
    }

    public void onExportClicked() {
        Stage stage = (Stage) containerAnchorPane.getScene().getWindow();
//        MessageUtils.saveMessage(getConnectionId(), messageDTO, stage);


        FileChooser fileChooser = new FileChooser();
        fileChooser.setTitle(resources.getString("exportUtilsTitle"));
        FileChooser.ExtensionFilter extFilter = new FileChooser.ExtensionFilter(resources.getString("exportUtilsDescription"), "*.json");
        fileChooser.getExtensionFilters().add(extFilter);

        File file = fileChooser.showSaveDialog(stage);
        if (file != null) {
            TaskFactory.exportConnection(null, file, connectionConfigDTOS);
        }
    }

    @FXML
    public void onCancelClicked() {
        Stage stage = (Stage) exportButton.getScene().getWindow();
        stage.close();
    }

}
