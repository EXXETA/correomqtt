package org.correomqtt.gui.views.connectionsettings;

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
import org.correomqtt.core.CoreManager;
import org.correomqtt.core.connection.DisconnectTaskFactory;
import org.correomqtt.core.keyring.KeyringFactory;
import org.correomqtt.core.model.ConnectionConfigDTO;
import org.correomqtt.core.mqtt.CorreoMqttClient;
import org.correomqtt.di.Assisted;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import org.correomqtt.gui.keyring.KeyringManager;
import org.correomqtt.gui.model.ConnectionPropertiesDTO;
import org.correomqtt.gui.model.WindowProperty;
import org.correomqtt.gui.model.WindowType;
import org.correomqtt.gui.theme.ThemeManager;
import org.correomqtt.gui.transformer.ConnectionTransformer;
import org.correomqtt.gui.utils.AlertHelper;
import org.correomqtt.gui.utils.WindowHelper;
import org.correomqtt.gui.views.LoaderResult;
import org.correomqtt.gui.views.base.BaseControllerImpl;
import org.correomqtt.gui.views.cell.ConnectionCell;
import org.correomqtt.gui.views.cell.ConnectionCellFactory;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.ResourceBundle;
import java.util.Set;
import java.util.UUID;

@DefaultBean
public class ConnectionSettingsViewController extends BaseControllerImpl {

    public static final String EXCLAMATION_CIRCLE_SOLID = "exclamationCircleSolid";
    private static final Logger LOGGER = LoggerFactory.getLogger(ConnectionSettingsViewController.class);
    private ResourceBundle resources;
    private final Map<String, ConnectionState> connectionStates = new HashMap<>();
    private final KeyringManager keyringManager;
    private final KeyringFactory keyringFactory;
    private final DisconnectTaskFactory disconnectTaskFactory;
    private final ConnectionCellFactory connectionCellFactory;
    private final AlertHelper alertHelper;
    private final MqttSettingsViewControllerFactory mqttSettingsViewControllerFactory;
    private final ConnectionPropertiesDTO preSelected;
    Set<String> waitForDisconnectIds = new HashSet<>();
    @FXML
    private VBox contentHolder;
    @FXML
    private Button discardButton;
    private ConnectionState connectionState;
    @FXML
    private VBox editConnectionContainer;

    @FXML
    private HBox emptyHint;
    @FXML
    private Label connectionSettingsViewHint;
    @FXML
    private HBox mainArea;
    @FXML
    private Button deleteButton;

    @FXML
    private ListView<ConnectionPropertiesDTO> connectionsListView;

    @FXML
    private Button saveButton;

    @FXML
    private AnchorPane containerAnchorPane;
    private boolean dragging;



    @Inject
    public ConnectionSettingsViewController(CoreManager coreManager,
                                            KeyringManager keyringManager,
                                            KeyringFactory keyringFactory,
                                            DisconnectTaskFactory disconnectTaskFactory,
                                            ThemeManager themeManager,
                                            ConnectionCellFactory connectionCellFactory,
                                            AlertHelper alertHelper,
                                            MqttSettingsViewControllerFactory mqttSettingsViewControllerFactory,
                                            @Assisted ConnectionPropertiesDTO preSelected) {
        super(coreManager, themeManager);
        this.keyringManager = keyringManager;
        this.keyringFactory = keyringFactory;
        this.disconnectTaskFactory = disconnectTaskFactory;
        this.connectionCellFactory = connectionCellFactory;
        this.alertHelper = alertHelper;
        this.mqttSettingsViewControllerFactory = mqttSettingsViewControllerFactory;
        this.preSelected = preSelected;
    }

    public void showAsDialog() {

        Map<Object, Object> properties = new HashMap<>();
        properties.put(WindowProperty.WINDOW_TYPE, WindowType.CONNECTION_SETTINGS);

        if (WindowHelper.focusWindowIfAlreadyThere(properties)) {
            return;
        }
        LoaderResult<ConnectionSettingsViewController> result = load();
        resources = result.getResourceBundle();

        showAsDialog(result, resources.getString("connectionSettingsViewControllerTitle"), properties, false, false,
                event -> result.getController().onCloseRequest(event),
                event -> result.getController().keyHandling(event),
                800, 500);

        result.getController().autoNew();
    }

    public LoaderResult<ConnectionSettingsViewController> load() {
        return load(ConnectionSettingsViewController.class, "connectionSettingsView.fxml",
                () -> this);
    }

    private void onCloseRequest(WindowEvent event) {
        // Window is only allowed to close if all connections are not dirty/new or the user discarded via alert.
        if (connectionsListView.getItems()
                .stream()
                .anyMatch(i -> i.isDirty() || i.isNew()) &&
                !alertHelper.confirm(
                        resources.getString("connectionSettingsViewControllerUnsavedTitle"),
                        resources.getString("connectionSettingsViewControllerUnsavedHeader"),
                        resources.getString("connectionSettingsViewControllerUnsavedContent"),
                        resources.getString("commonCancelButton"),
                        resources.getString("commonDiscardButton")
                )) {
            event.consume();
        }
    }

    private void keyHandling(KeyEvent event) {
        if (KeyCode.ESCAPE == event.getCode()) {
            closeDialog();
        }
    }

    private void autoNew() {
        if (connectionsListView.getItems().isEmpty()) {
            addConnection();
        }
    }

    private void closeDialog() {
        Stage stage = (Stage) mainArea.getScene().getWindow();
        stage.close();
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
            LoaderResult<MqttSettingsViewController> result = mqttSettingsViewControllerFactory.create().load();
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

        String keyringName = resources.getString(keyringFactory.createKeyringByIdentifier(coreManager.getSettingsManager()
                .getSettings()
                .getKeyringIdentifier()).getName());

        connectionSettingsViewHint.setText(resources.getString("connectionSettingsViewHint") + " (" + keyringName + ").");
    }

    private void onDirtyChanged(Observable observable) {
        boolean dirty = connectionState.controller.getDTO().isDirty();
        boolean isnew = connectionState.controller.getDTO().isNew();
        this.discardButton.setDisable(!dirty && !isnew);
        this.saveButton.setDisable(!dirty && !isnew);
        this.deleteButton.setDisable(isnew);
    }

    @FXML
    private void initialize() {
        containerAnchorPane.getStyleClass().add(themeManager.getIconModeCssClass());
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

    private void loadConnectionListFromBackground() {
        ObservableList<ConnectionPropertiesDTO> list = FXCollections.observableArrayList(ConnectionPropertiesDTO.extractor());
        coreManager.getConnectionManager().getSortedConnections()
                .forEach(c -> list.add(ConnectionTransformer.dtoToProps(c)));
        connectionsListView.setItems(list);
        LOGGER.debug("Loading connection list from background");
    }

    private ListCell<ConnectionPropertiesDTO> createCell(ListView<ConnectionPropertiesDTO> connectionListView) {
        ConnectionCell cell = connectionCellFactory.create(connectionListView);
        cell.selectedProperty().addListener(this::onCellSelected);

        setOnDragDetected(cell);
        setOnDragOver(cell);
        setOnDragEntered(cell);
        setOnDragDropped(cell);
        cell.setOnDragDone(DragEvent::consume);

        return cell;
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

    private void setOnDragDropped(ConnectionCell cell) {
        cell.setOnDragDropped(event -> {
            event.setDropCompleted(true);
            dragging = false;
            LOGGER.info("Drag Dropped");
            persistConnections(null, false);
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

    private void persistConnections(ConnectionPropertiesDTO config, boolean delete) {


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
                        return coreManager.getConnectionManager().getConfig(p.getId());
                    }
                })
                .toList();


        keyringManager.retryWithMasterPassword(
                masterPassword -> coreManager.getSettingsManager().saveConnections(connectionsToSave, masterPassword),
                resources.getString("onPasswordSaveFailedTitle"),
                resources.getString("onPasswordSaveFailedHeader"),
                resources.getString("onPasswordSaveFailedContent"),
                resources.getString("onPasswordSaveFailedGiveUp"),
                resources.getString("onPasswordSaveFailedTryAgain")
        );
    }

    @FXML
    private void onAddClicked() {
        LOGGER.debug("Add new connection clicked");

        addConnection();
    }

    @FXML
    private void onRemoveClicked() {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Remove connection clicked: {}", connectionState.controller.getDTO().getId());
        }

        if (isSafeToDiscard()) {
            dropConnection();
        }
    }

    private boolean isSafeToDiscard() {
        if (connectionState != null && (
                connectionState.controller.getDTO().isDirty() ||
                        connectionState.controller.getDTO().isNew())) {
            if (alertHelper.confirm(
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

    private void dropConnection() {

        ConnectionPropertiesDTO config = connectionState.controller.getDTO();

        boolean confirmed = alertHelper.confirm(
                resources.getString("connectionSettingsViewControllerDeleteTitle"),
                resources.getString("connectionSettingsViewControllerDeleteHeader") + "? (" + config + ")",
                null,
                resources.getString("commonNoButton"),
                resources.getString("commonYesButton")
        );


        if (confirmed) {

            LOGGER.info("Disconnect connection selected");

            CorreoMqttClient client = coreManager.getConnectionManager().getConnection(config.getId()).getClient();
            if (client != null) {
                LOGGER.info("Connection is still connected");

                confirmed = alertHelper.confirm(
                        resources.getString("connectionSettingsViewControllerStillInUseTitle"),
                        resources.getString("connectionSettingsViewControllerStillInUseHeader"),
                        null,
                        resources.getString("commonNoButton"),
                        resources.getString("commonYesButton")
                );

                if (confirmed) {
                    LOGGER.info("Disconnect");
                    disconnectTaskFactory.create(config.getId()).run();
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

    @FXML
    private void onDiscardClicked() {
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
    private void onSaveClicked() {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Save changes clicked: {}", connectionState.controller.getDTO().getId());
        }
        if (connectionState.controller.saveConnection()) {
            deleteButton.setDisable(false);
            persistConnections(connectionState.controller.getDTO(), false);
            onDirtyChanged(null);
        }
    }

    @AllArgsConstructor
    private static class ConnectionState {
        private ConnectionSettingsDelegateController controller;
        private Region region;
    }

}