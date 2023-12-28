package org.correomqtt.gui.controller;

import javafx.beans.Observable;
import javafx.beans.value.ObservableValue;
import javafx.collections.FXCollections;
import javafx.collections.ObservableList;
import javafx.fxml.FXML;
import javafx.scene.control.Button;
import javafx.scene.control.Label;
import javafx.scene.control.ListCell;
import javafx.scene.control.ListView;
import javafx.scene.input.ClipboardContent;
import javafx.scene.input.DragEvent;
import javafx.scene.input.Dragboard;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.TransferMode;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.HBox;
import javafx.scene.layout.Region;
import javafx.scene.layout.VBox;
import javafx.stage.Stage;
import javafx.stage.WindowEvent;
import lombok.AllArgsConstructor;
import org.correomqtt.business.dispatcher.ConfigDispatcher;
import org.correomqtt.business.dispatcher.ConfigObserver;
import org.correomqtt.business.keyring.KeyringFactory;
import org.correomqtt.business.model.ConnectionConfigDTO;
import org.correomqtt.business.mqtt.CorreoMqttClient;
import org.correomqtt.business.provider.SettingsProvider;
import org.correomqtt.business.utils.ConnectionHolder;
import org.correomqtt.gui.business.MessageTaskFactory;
import org.correomqtt.gui.cell.ConnectionCell;
import org.correomqtt.gui.helper.AlertHelper;
import org.correomqtt.gui.keyring.KeyringHandler;
import org.correomqtt.gui.model.ConnectionPropertiesDTO;
import org.correomqtt.gui.model.WindowProperty;
import org.correomqtt.gui.model.WindowType;
import org.correomqtt.gui.transformer.ConnectionTransformer;
import org.correomqtt.gui.utils.WindowHelper;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.ResourceBundle;
import java.util.Set;
import java.util.UUID;

public class ConnectionSettingsViewController extends BaseControllerImpl implements ConfigObserver {

    @FXML
    public VBox contentHolder;

    @FXML
    public Button discardButton;
    private ConnectionState connectionState;

    private void onCloseRequest(WindowEvent event) {
        // Window is only allowed to close if all connections are not dirty/new or the user discarded via alert.
        if (connectionsListView.getItems()
                .stream()
                .anyMatch(i -> i.isDirty() || i.isNew()) &&
                !AlertHelper.confirm(
                        resources.getString("connectionSettingsViewControllerUnsavedTitle"),
                        resources.getString("connectionSettingsViewControllerUnsavedHeader"),
                        resources.getString("connectionSettingsViewControllerUnsavedContent"),
                        resources.getString("commonCancelButton"),
                        resources.getString("commonDiscardButton")
                )) {
            event.consume();
        } else {
            cleanUp();
        }
    }

    @AllArgsConstructor
    private static class ConnectionState {
        private ConnectionSettingsDelegateController controller;
        private Region region;
    }

    private static final Logger LOGGER = LoggerFactory.getLogger(ConnectionSettingsViewController.class);

    public static final String EXCLAMATION_CIRCLE_SOLID = "exclamationCircleSolid";

    private final ConnectionSettingsViewDelegate delegate;
    private final Map<String, ConnectionState> connectionStates = new HashMap<>();

    private final ConnectionPropertiesDTO preSelected;

    @FXML
    public VBox editConnectionContainer;

    @FXML
    public HBox emptyHint;
    @FXML
    public Label connectionSettingsViewHint;
    @FXML
    public HBox mainArea;
    @FXML
    public Button deleteButton;

    @FXML
    private ListView<ConnectionPropertiesDTO> connectionsListView;

    @FXML
    private Button saveButton;

    @FXML
    private AnchorPane containerAnchorPane;

    private static ResourceBundle resources;

    private boolean dragging;

    Set<String> waitForDisconnectIds = new HashSet<>();

    public ConnectionSettingsViewController(ConnectionSettingsViewDelegate delegate, ConnectionPropertiesDTO preSelected) {
        this.delegate = delegate;
        this.preSelected = preSelected;
        ConfigDispatcher.getInstance().addObserver(this);
    }

    public static LoaderResult<ConnectionSettingsViewController> load(ConnectionSettingsViewDelegate delegate, ConnectionPropertiesDTO preSelected) {
        return load(ConnectionSettingsViewController.class, "connectionSettingsView.fxml",
                () -> new ConnectionSettingsViewController(delegate, preSelected));
    }

    public static void showAsDialog(ConnectionSettingsViewDelegate delegate, ConnectionPropertiesDTO preSelected) {

        Map<Object, Object> properties = new HashMap<>();
        properties.put(WindowProperty.WINDOW_TYPE, WindowType.CONNECTION_SETTINGS);

        if (WindowHelper.focusWindowIfAlreadyThere(properties)) {
            return;
        }
        LoaderResult<ConnectionSettingsViewController> result = load(delegate, preSelected);
        resources = result.getResourceBundle();

        showAsDialog(result, resources.getString("connectionSettingsViewControllerTitle"), properties, false, false,
                event -> result.getController().onCloseRequest(event),
                event -> result.getController().keyHandling(event),
                800, 500);

        result.getController().autoNew();
    }

    private void autoNew() {
        if (connectionsListView.getItems().isEmpty()) {
            addConnection();
        }
    }

    @FXML
    public void initialize() {
        containerAnchorPane.getStyleClass().add(SettingsProvider.getInstance().getIconModeCssClass());
        loadConnectionListFromBackground();
        connectionsListView.setCellFactory(this::createCell);
        deselectConnection();
        if (preSelected != null) {
            connectionsListView.getSelectionModel().select(connectionsListView.getItems()
                    .stream()
                    .filter(i -> i.getId().equals(preSelected.getId()))
                    .findFirst()
                    .orElse(null));
        } else {
            connectionsListView.getSelectionModel().select(connectionsListView.getItems()
                    .stream()
                    .findFirst()
                    .orElse(null));
        }
    }

    private void deselectConnection() {
        if (connectionState != null) {
            connectionState.controller.cleanUp();
            editConnectionContainer.getChildren().remove(connectionState.region);
            connectionState = null;
        }
        if (connectionsListView.getItems().isEmpty()) {
            contentHolder.getChildren().remove(mainArea);
            if (!contentHolder.getChildren().contains(emptyHint)) {
                contentHolder.getChildren().add(emptyHint);
            }
        } else {
            connectionsListView.getSelectionModel().selectFirst();
        }
    }

    private ListCell<ConnectionPropertiesDTO> createCell(ListView<ConnectionPropertiesDTO> connectionListView) {
        ConnectionCell cell = new ConnectionCell(connectionListView);
        cell.selectedProperty().addListener(this::onCellSelected);

        setOnDragDetected(cell);
        setOnDragOver(cell);
        setOnDragEntered(cell);
        setOnDragDropped(cell);
        cell.setOnDragDone(DragEvent::consume);

        return cell;
    }

    private void onCellSelected(ObservableValue<? extends Boolean> observable, Boolean oldValue, Boolean newValue) {

        if (Boolean.FALSE.equals(newValue) || dragging) {
            return;
        }

        ConnectionPropertiesDTO selectedItem = connectionsListView.getSelectionModel().getSelectedItem();

        if (selectedItem != null) {
            showConnection(selectedItem);
        }
    }

    private void setOnDragDetected(ConnectionCell cell) {
        cell.setOnDragDetected(event -> {
            LOGGER.info("Drag detected");
            if (cell.getItem() == null) {
                return;
            }

            dragging = true;

            Dragboard dragboard = cell.startDragAndDrop(TransferMode.MOVE);
            ClipboardContent content = new ClipboardContent();
            content.putString(String.valueOf(connectionsListView.getSelectionModel().getSelectedIndex()));
            dragboard.setContent(content);

            event.consume();
        });
    }

    private void setOnDragOver(ConnectionCell cell) {
        cell.setOnDragOver(event -> {
            if (event.getDragboard().hasString()) {
                event.acceptTransferModes(TransferMode.MOVE);
            }
            event.consume();
        });
    }

    private void setOnDragEntered(ConnectionCell cell) {
        cell.setOnDragEntered(event -> {
            if (event.getDragboard().hasString()) {
                Dragboard db = event.getDragboard();
                if (db.hasString()) {
                    ObservableList<ConnectionPropertiesDTO> items = cell.getListView().getItems();
                    performDrag(connectionsListView.getSelectionModel().getSelectedIndex(), items.indexOf(cell.getItem()));
                }
            }
        });
    }

    private void performDrag(int draggedIdx, int thisIdx) {
        if (draggedIdx > thisIdx && thisIdx > -1) {
            for (int i = draggedIdx; i > thisIdx; i--) {
                ConnectionPropertiesDTO temp = connectionsListView.getItems().get(i);
                connectionsListView.getItems().set(i, connectionsListView.getItems().get(i - 1));
                connectionsListView.getItems().set(i - 1, temp);
            }
        }

        if (draggedIdx < thisIdx && draggedIdx > -1) {
            for (int i = draggedIdx; i < thisIdx; i++) {
                ConnectionPropertiesDTO temp = connectionsListView.getItems().get(i);
                connectionsListView.getItems().set(i, connectionsListView.getItems().get(i + 1));
                connectionsListView.getItems().set(i + 1, temp);
            }
        }

        connectionsListView.getSelectionModel().select(thisIdx);
    }

    private void setOnDragDropped(ConnectionCell cell) {
        cell.setOnDragDropped(event -> {
            event.setDropCompleted(true);
            dragging = false;
            LOGGER.info("Drag Dropped");
            persistConnections(null, false);
        });
    }

    private void loadConnectionListFromBackground() {
        ObservableList<ConnectionPropertiesDTO> list = FXCollections.observableArrayList(ConnectionPropertiesDTO.extractor());
        ConnectionHolder.getInstance().getSortedConnections()
                .forEach(c -> list.add(ConnectionTransformer.dtoToProps(c)));
        connectionsListView.setItems(list);
        LOGGER.debug("Loading connection list from background");
    }

    private boolean isSafeToDiscard() {
        if (connectionState != null && (
                connectionState.controller.getDTO().isDirty() ||
                        connectionState.controller.getDTO().isNew())) {
            if (AlertHelper.confirm(
                    resources.getString("connectionSettingsViewControllerUnsavedTitle"),
                    resources.getString("connectionSettingsViewControllerUnsavedHeader"),
                    resources.getString("connectionSettingsViewControllerUnsavedContent"),
                    resources.getString("commonCancelButton"),
                    resources.getString("commonDiscardButton")
            )) {
                connectionState.controller.resetDTO();
                return true;
            }
            return false;
        }
        return true;
    }


    @FXML
    public void onAddClicked() {
        LOGGER.debug("Add new connection clicked");

        addConnection();
    }

    @FXML
    public void onRemoveClicked() {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Remove connection clicked: {}", connectionState.controller.getDTO().getId());
        }

        if (isSafeToDiscard()) {
            dropConnection();
        }
    }

    @FXML
    public void onDiscardClicked() {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Discard editing clicked.");
        }
        if (isSafeToDiscard()) {
            if (connectionState.controller.getDTO().isNew()) {
                connectionsListView.getItems().remove(connectionState.controller.getDTO());
            }
            deselectConnection();
        }
    }

    @FXML
    public void onSaveClicked() {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Save changes clicked: {}", connectionState.controller.getDTO().getId());
        }
        if (connectionState.controller.saveConnection()) {
            deleteButton.setDisable(false);
            persistConnections(connectionState.controller.getDTO(), false);
            onDirtyChanged(null);
        }
    }

    private void persistConnections(ConnectionPropertiesDTO config, boolean delete) {

        ConnectionHolder ch = ConnectionHolder.getInstance();

        // get connections in order
        List<ConnectionConfigDTO> connectionsToSave = connectionsListView.getItems()
                .stream()

                // skip if config should be deleted or config is new and not yet saved.
                .filter(p -> (p == config && !delete) || (p != config && !p.isNew()))

                // get original configs to ignore dirty except the one that should be changed
                .map(p -> {
                    if (p == config) {
                        return ConnectionTransformer.propsToDto(p);
                    } else {
                        return ch.getConfig(p.getId());
                    }
                })
                .toList();


        KeyringHandler.getInstance().retryWithMasterPassword(
                masterPassword -> SettingsProvider.getInstance().saveConnections(connectionsToSave, masterPassword),
                resources.getString("onPasswordSaveFailedTitle"),
                resources.getString("onPasswordSaveFailedHeader"),
                resources.getString("onPasswordSaveFailedContent"),
                resources.getString("onPasswordSaveFailedGiveUp"),
                resources.getString("onPasswordSaveFailedTryAgain")
        );
    }


    private void closeDialog() {
        cleanUp();
        Stage stage = (Stage) mainArea.getScene().getWindow();
        stage.close();
    }

    private void keyHandling(KeyEvent event) {
        if (KeyCode.ESCAPE == event.getCode()) {
            closeDialog();
        }
    }

    private void showConnection(ConnectionPropertiesDTO config) {

        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Show connection: {}", config.getId());
        }

        if (connectionState != null) {
            connectionState.controller.getDTO().getDirtyProperty().removeListener(this::onDirtyChanged);
            connectionState.controller.cleanUp();
            editConnectionContainer.getChildren().remove(connectionState.region);
        }

        boolean init = !connectionStates.containsKey(config.getId());

        connectionState = connectionStates.computeIfAbsent(config.getId(), id -> {
            // TODO for Kafka: Use Factory
            LoaderResult<MqttSettingsViewController> result = MqttSettingsViewController.load();
            return new ConnectionState(result.getController(), result.getMainRegion());
        });

        if (init) {
            connectionState.controller.setDTO(config);
        }
        connectionState.controller.getDTO().getDirtyProperty().addListener(this::onDirtyChanged);
        contentHolder.getChildren().remove(emptyHint);
        if (!contentHolder.getChildren().contains(mainArea)) {
            contentHolder.getChildren().add(mainArea);
        }
        editConnectionContainer.getChildren().add(0, connectionState.region);
        onDirtyChanged(null);

        String keyringName =
                KeyringFactory.createKeyringByIdentifier(SettingsProvider.getInstance().getSettings().getKeyringIdentifier()).getName();

        connectionSettingsViewHint.setText(resources.getString("connectionSettingsViewHint") + " (" + keyringName + ").");
    }

    private void onDirtyChanged(Observable observable) {
        boolean dirty = connectionState.controller.getDTO().isDirty();
        boolean isnew = connectionState.controller.getDTO().isNew();
        this.discardButton.setDisable(!dirty && !isnew);
        this.saveButton.setDisable(!dirty && !isnew);
        this.deleteButton.setDisable(isnew);
    }

    private void addConnection() {

        String newConnectionId = UUID.randomUUID().toString();

        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("New Connection: {}", newConnectionId);
        }

        ConnectionPropertiesDTO newConfig = ConnectionTransformer.dtoToProps(ConnectionConfigDTO.builder()
                .id(newConnectionId)
                .name(resources.getString(
                        "connectionSettingsViewControllerNewConnectionName"))
                .build());

        newConfig.getUnpersistedProperty().set(true);
        newConfig.getDirtyProperty().set(true);
        newConfig.getNewProperty().set(true);

        connectionsListView.getItems().add(newConfig);
        connectionsListView.getSelectionModel().select(newConfig);
        newConfig.getUnpersistedProperty().set(true);
        showConnection(newConfig);
    }

    private void dropConnection() {

        ConnectionPropertiesDTO config = connectionState.controller.getDTO();

        boolean confirmed = AlertHelper.confirm(
                resources.getString("connectionSettingsViewControllerDeleteTitle"),
                resources.getString("connectionSettingsViewControllerDeleteHeader") + "? (" + config + ")",
                null,
                resources.getString("commonNoButton"),
                resources.getString("commonYesButton")
        );


        if (confirmed) {

            LOGGER.info("Disconnect connection selected");

            CorreoMqttClient client = ConnectionHolder.getInstance().getConnection(config.getId()).getClient();
            if (client != null) {
                LOGGER.info("Connection is still connected");

                confirmed = AlertHelper.confirm(
                        resources.getString("connectionSettingsViewControllerStillInUseTitle"),
                        resources.getString("connectionSettingsViewControllerStillInUseHeader"),
                        null,
                        resources.getString("commonNoButton"),
                        resources.getString("commonYesButton")
                );

                if (confirmed) {
                    LOGGER.info("Disconnect");
                    MessageTaskFactory.disconnect(config.getId());
                    waitForDisconnectIds.add(config.getId());
                }
            } else {
                dropConnectionForReal(config);
            }
        }
    }

    private void dropConnectionForReal(ConnectionPropertiesDTO config) {
        connectionsListView.getItems().remove(config);
        persistConnections(config, true);
        deselectConnection();
    }

    @Override
    public void onConfigDirectoryEmpty() {
        // Do nothing
    }

    @Override
    public void onConfigDirectoryNotAccessible() {
        // Do nothing
    }

    @Override
    public void onAppDataNull() {
        // Do nothing
    }

    @Override
    public void onUserHomeNull() {
        // Do nothing
    }

    @Override
    public void onFileAlreadyExists() {
        // Do nothing
    }

    @Override
    public void onInvalidPath() {
        // Do nothing
    }

    @Override
    public void onInvalidJsonFormat() {
        // Do nothing
    }

    @Override
    public void onSavingFailed() {
        // Do nothing
    }

    @Override
    public void onSettingsUpdated(boolean showRestartRequiredDialog) {

        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Updated settings in connection settings view controller");
        }
    }

    @Override
    public void onConnectionsUpdated() {
        // Do nothing
    }

    @Override
    public void onConfigPrepareFailed() {
        // Do nothing
    }

    public void cleanUp() {
        ConfigDispatcher.getInstance().removeObserver(this);
    }

}