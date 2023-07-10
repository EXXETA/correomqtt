package org.correomqtt.gui.controller;

import javafx.collections.FXCollections;
import javafx.collections.ObservableList;
import javafx.fxml.FXML;
import javafx.scene.control.Button;
import javafx.scene.control.CheckBox;
import javafx.scene.control.ComboBox;
import javafx.scene.control.Label;
import javafx.scene.control.ListCell;
import javafx.scene.control.ListView;
import javafx.scene.control.PasswordField;
import javafx.scene.control.TabPane;
import javafx.scene.control.TextField;
import javafx.scene.control.Tooltip;
import javafx.scene.input.ClipboardContent;
import javafx.scene.input.DragEvent;
import javafx.scene.input.Dragboard;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.TransferMode;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.GridPane;
import javafx.scene.layout.HBox;
import javafx.scene.layout.Pane;
import javafx.scene.layout.VBox;
import javafx.stage.FileChooser;
import javafx.stage.Stage;
import javafx.util.StringConverter;
import org.correomqtt.business.dispatcher.ConfigDispatcher;
import org.correomqtt.business.dispatcher.ConfigObserver;
import org.correomqtt.business.dispatcher.ConnectionLifecycleDispatcher;
import org.correomqtt.business.dispatcher.ConnectionLifecycleObserver;
import org.correomqtt.business.dispatcher.ImportConnectionDispatcher;
import org.correomqtt.business.dispatcher.ImportConnectionObserver;
import org.correomqtt.business.keyring.KeyringFactory;
import org.correomqtt.business.model.Auth;
import org.correomqtt.business.model.ConnectionConfigDTO;
import org.correomqtt.business.model.ConnectionExportDTO;
import org.correomqtt.business.model.CorreoMqttVersion;
import org.correomqtt.business.model.GenericTranslatable;
import org.correomqtt.business.model.Lwt;
import org.correomqtt.business.model.Proxy;
import org.correomqtt.business.model.Qos;
import org.correomqtt.business.model.TlsSsl;
import org.correomqtt.business.mqtt.CorreoMqttClient;
import org.correomqtt.business.provider.SettingsProvider;
import org.correomqtt.business.utils.ConnectionHolder;
import org.correomqtt.gui.business.MessageTaskFactory;
import org.correomqtt.gui.cell.ConnectionCell;
import org.correomqtt.gui.cell.GenericCell;
import org.correomqtt.gui.helper.AlertHelper;
import org.correomqtt.gui.helper.CheckTopicHelper;
import org.correomqtt.gui.keyring.KeyringHandler;
import org.correomqtt.gui.model.ConnectionPropertiesDTO;
import org.correomqtt.gui.model.WindowProperty;
import org.correomqtt.gui.model.WindowType;
import org.correomqtt.gui.transformer.ConnectionTransformer;
import org.correomqtt.gui.utils.WindowHelper;
import org.correomqtt.plugin.manager.PluginManager;
import org.correomqtt.plugin.model.LwtConnectionExtensionDTO;
import org.correomqtt.plugin.spi.LwtSettingsHook;
import org.fxmisc.flowless.VirtualizedScrollPane;
import org.fxmisc.richtext.CodeArea;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.File;
import java.util.Base64;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.ResourceBundle;
import java.util.UUID;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicInteger;

public class ConnectionSettingsViewController extends BaseController implements ConfigObserver, ImportConnectionObserver, ConnectionLifecycleObserver,
        LwtSettingsHook.OnSettingsChangedListener {

    private static final String TEXT_FIELD = "text-field";
    private static final String TEXT_INPUT = "text-input";
    private static final Logger LOGGER = LoggerFactory.getLogger(ConnectionSettingsViewController.class);
    private static final int CLIENT_ID_MAX_SIZE = 64;
    public static final String EXCLAMATION_CIRCLE_SOLID = "exclamationCircleSolid";
    public static final String EMPTY_ERROR_CLASS = "emptyError";
    private final ConnectionSettingsViewDelegate delegate;
    private final ConnectionExportViewDelegate connectionExportViewDelegate;
    private final ConnectionImportViewDelegate connectionImportViewDelegate;

    @FXML
    private ListView<ConnectionPropertiesDTO> connectionsListView;
    @FXML
    private TabPane connectionConfigTabPane;
    @FXML
    private TextField nameTextField;
    @FXML
    private TextField urlTextField;
    @FXML
    private TextField portTextField;
    @FXML
    private TextField clientIdTextField;
    @FXML
    private TextField usernameTextField;
    @FXML
    private PasswordField passwordField;
    @FXML
    private CheckBox cleanSessionCheckBox;
    @FXML
    private ComboBox<CorreoMqttVersion> mqttVersionComboBox;
    @FXML
    private Label internalIdLabel;
    @FXML
    private ComboBox<TlsSsl> tlsComboBox;
    @FXML
    private GridPane tlsSslGridPane;
    @FXML
    private TextField sslKeystoreTextField;
    @FXML
    private TextField sslKeystorePasswordTextField;
    @FXML
    private ComboBox<Proxy> proxyComboBox;
    @FXML
    private GridPane proxyGridPane;
    @FXML
    private TextField sshHostTextField;
    @FXML
    private TextField sshPortTextField;
    @FXML
    private TextField localPortTextField;
    @FXML
    private ComboBox<Auth> authComboBox;
    @FXML
    private TextField authUsernameTextField;
    @FXML
    private PasswordField authPasswordField;
    @FXML
    private HBox authKeyfileHBox;
    @FXML
    private TextField authKeyFileTextField;
    @FXML
    private Label dropLabel;
    @FXML
    private Label upLabel;
    @FXML
    private Label downLabel;
    @FXML
    private Label exportLabel;
    @FXML
    private Label importLabel;
    @FXML
    private Button applyButton;
    @FXML
    private Button saveButton;
    @FXML
    private AnchorPane containerAnchorPane;
    @FXML
    private VBox lwtContentVBox;
    @FXML
    private ComboBox<Lwt> lwtComboBox;
    @FXML
    private ComboBox<String> lwtTopicComboBox;
    @FXML
    private ComboBox<Qos> lwtQoSComboBox;
    @FXML
    private HBox lwtPluginControlBox;
    @FXML
    private CheckBox lwtRetainedCheckBox;
    @FXML
    private CodeArea lwtPayloadCodeArea;
    @FXML
    private Pane lwtPayloadPane;
    @FXML
    private Label connectionSettingsViewHint;

    private static ResourceBundle resources;

    private ConnectionPropertiesDTO activeConnectionConfigDTO;

    private AtomicBoolean dirtyCheckEnabled = new AtomicBoolean(true);
    private boolean dragging;
    Map<String, Integer> waitForDisconnectIds = new HashMap<>();

    public ConnectionSettingsViewController(ConnectionSettingsViewDelegate delegate, ConnectionExportViewDelegate exportViewDelegate, ConnectionImportViewDelegate importViewDelegate) {
        this.delegate = delegate;
        this.connectionExportViewDelegate = exportViewDelegate;
        this.connectionImportViewDelegate = importViewDelegate;
        ConfigDispatcher.getInstance().addObserver(this);
        ConnectionLifecycleDispatcher.getInstance().addObserver(this);
        ImportConnectionDispatcher.getInstance().addObserver(this);

    }

    public static LoaderResult<ConnectionSettingsViewController> load(ConnectionSettingsViewDelegate delegate, ConnectionExportViewDelegate exportViewDelegate, ConnectionImportViewDelegate importViewDelegate) {
        return load(ConnectionSettingsViewController.class, "connectionSettingsView.fxml",
                () -> new ConnectionSettingsViewController(delegate, exportViewDelegate, importViewDelegate));
    }


    public static void showAsDialog(ConnectionSettingsViewDelegate delegate, ConnectionExportViewDelegate exportViewDelegate, ConnectionImportViewDelegate importViewDelegate) {


        Map<Object, Object> properties = new HashMap<>();
        properties.put(WindowProperty.WINDOW_TYPE, WindowType.CONNECTION_SETTINGS);

        if (WindowHelper.focusWindowIfAlreadyThere(properties)) {
            return;
        }
        LoaderResult<ConnectionSettingsViewController> result = load(delegate, exportViewDelegate, importViewDelegate);
        resources = result.getResourceBundle();

        showAsDialog(result, resources.getString("connectionSettingsViewControllerTitle"), properties, false, false, null,
                event -> result.getController().keyHandling(event));
    }

    @FXML
    public void initialize() {

        containerAnchorPane.getStyleClass().add(SettingsProvider.getInstance().getIconModeCssClass());

        connectionConfigTabPane.setDisable(true);
        loadConnectionListFromBackground();

        mqttVersionComboBox.setItems(FXCollections.observableArrayList(CorreoMqttVersion.values()));
        mqttVersionComboBox.setCellFactory(GenericCell::new);
        mqttVersionComboBox.setConverter(getStringConverter());

        tlsComboBox.setItems(FXCollections.observableArrayList(TlsSsl.values()));
        tlsComboBox.setCellFactory(GenericCell::new);
        tlsComboBox.setConverter(getStringConverter());

        proxyComboBox.setItems(FXCollections.observableArrayList(Proxy.values()));
        proxyComboBox.setCellFactory(GenericCell::new);
        proxyComboBox.setConverter(getStringConverter());

        authComboBox.setItems(FXCollections.observableArrayList(Auth.values()));
        authComboBox.setCellFactory(GenericCell::new);
        authComboBox.setConverter(getStringConverter());

        lwtComboBox.setItems(FXCollections.observableArrayList(Lwt.values()));
        lwtComboBox.setCellFactory(GenericCell::new);
        lwtComboBox.setConverter(getStringConverter());

        lwtQoSComboBox.setItems(FXCollections.observableArrayList(Qos.values()));

        lwtPayloadPane.getChildren().add(new VirtualizedScrollPane<>(lwtPayloadCodeArea));
        lwtPayloadCodeArea.prefWidthProperty().bind(lwtPayloadPane.widthProperty());
        lwtPayloadCodeArea.prefHeightProperty().bind(lwtPayloadPane.heightProperty());

        PluginManager.getInstance().getExtensions(LwtSettingsHook.class).forEach(p -> p.onAddItemsToLwtSettingsBox(this, lwtPluginControlBox));

        connectionsListView.setCellFactory(this::createCell);

        saveButton.setDisable(true);
        applyButton.setDisable(true);
        dropLabel.setDisable(true);
        upLabel.setDisable(true);
        downLabel.setDisable(true);

        nameTextField.lengthProperty().addListener((observable, oldValue, newValue) ->
                checkName(nameTextField, false));
        urlTextField.lengthProperty().addListener(((observable, oldValue, newValue) ->
                checkUrl(urlTextField, false)));
        portTextField.lengthProperty().addListener(((observable, oldValue, newValue) ->
                checkPort(portTextField, false)));
        clientIdTextField.lengthProperty().addListener(((observable, oldValue, newValue) ->
                checkClientID(clientIdTextField, false)));

        nameTextField.textProperty().addListener((observable, oldValue, newValue) -> setDirty(true));
        urlTextField.textProperty().addListener((observable, oldValue, newValue) -> setDirty(true));
        portTextField.textProperty().addListener((observable, oldValue, newValue) -> setDirty(true));
        clientIdTextField.textProperty().addListener((observable, oldValue, newValue) -> setDirty(true));
        passwordField.textProperty().addListener((observable, oldValue, newValue) -> setDirty(true));
        usernameTextField.textProperty().addListener((observable, oldValue, newValue) -> setDirty(true));
        cleanSessionCheckBox.selectedProperty().addListener((observable, oldValue, newValue) -> setDirty(true));
        mqttVersionComboBox.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> setDirty(true));
        tlsComboBox.getSelectionModel().selectedItemProperty().addListener(((observable, oldValue, newValue) -> {
            setDirty(true);
            if (newValue.equals(TlsSsl.OFF)) {
                tlsSslGridPane.setDisable(true);
            } else if (newValue.equals(TlsSsl.KEYSTORE)) {
                tlsSslGridPane.setDisable(false);
            }
        }));
        sslKeystoreTextField.textProperty().addListener((observable, oldValue, newValue) -> setDirty(true));
        sslKeystorePasswordTextField.textProperty().addListener((observable, oldValue, newValue) -> setDirty(true));
        proxyComboBox.getSelectionModel().selectedItemProperty().addListener(((observable, oldValue, newValue) -> {
            setDirty(true);
            if (newValue.equals(Proxy.OFF)) {
                proxyGridPane.setDisable(true);
            } else if (newValue.equals(Proxy.SSH)) {
                proxyGridPane.setDisable(false);
            }
        }));
        sshHostTextField.textProperty().addListener((observable, oldValue, newValue) -> setDirty(true));
        sshPortTextField.textProperty().addListener((observable, oldValue, newValue) -> setDirty(true));
        localPortTextField.textProperty().addListener((observable, oldValue, newValue) -> setDirty(true));
        authComboBox.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> {
            setDirty(true);
            if (newValue.equals(Auth.OFF)) {
                authUsernameTextField.setDisable(true);
                authPasswordField.setDisable(true);
                authKeyfileHBox.setDisable(true);
            } else if (newValue.equals(Auth.PASSWORD)) {
                authUsernameTextField.setDisable(false);
                authPasswordField.setDisable(false);
                authKeyfileHBox.setDisable(true);
            } else if (newValue.equals(Auth.KEYFILE)) {
                authUsernameTextField.setDisable(false);
                authPasswordField.setDisable(true);
                authKeyfileHBox.setDisable(false);
            }
        });
        authUsernameTextField.textProperty().addListener(((observable, oldValue, newValue) -> setDirty(true)));
        authPasswordField.textProperty().addListener((observable, oldValue, newValue) -> setDirty(true));
        authKeyFileTextField.textProperty().addListener((observable, oldValue, newValue) -> setDirty(true));
        lwtComboBox.getSelectionModel().selectedItemProperty().addListener(((observable, oldValue, newValue) -> {
            setDirty(true);
            if (newValue.equals(Lwt.OFF)) {
                lwtContentVBox.setDisable(true);
            } else if (newValue.equals(Lwt.ON)) {
                lwtContentVBox.setDisable(false);
            }
        }));
        lwtTopicComboBox.getEditor().lengthProperty().addListener(((observable, oldValue, newValue) -> {
            setDirty(true);
            CheckTopicHelper.checkPublishTopic(lwtTopicComboBox, false);
        }));
        lwtQoSComboBox.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> setDirty(true));
        lwtRetainedCheckBox.selectedProperty().addListener((observable, oldValue, newValue) -> setDirty(true));
        lwtPayloadCodeArea.textProperty().addListener(((observable, oldValue, newValue) -> setDirty(true)));

        internalIdLabel.setText("");

        String keyringName = KeyringFactory.createKeyringByIdentifier(SettingsProvider.getInstance().getSettings().getKeyringIdentifier()).getName();
        connectionSettingsViewHint.setText(connectionSettingsViewHint.getText() + " (" + keyringName + ").");
    }

    private <T extends GenericTranslatable> StringConverter<T> getStringConverter() {
        return new StringConverter<T>() {
            @Override
            public String toString(T object) {
                if (object == null) {
                    return null;
                }
                String translationKey = object.getLabelTranslationKey();
                if (translationKey != null) {
                    return resources.getString(translationKey);
                }
                return object.toString();
            }

            @Override
            public T fromString(String string) {
                return null;
            }
        };
    }

    private ListCell<ConnectionPropertiesDTO> createCell(ListView<ConnectionPropertiesDTO> connectionListView) {
        ConnectionCell cell = new ConnectionCell(connectionListView);
        cell.selectedProperty().addListener((observable, oldValue, newValue) -> {

            ConnectionPropertiesDTO selectedItem = connectionsListView.getSelectionModel().getSelectedItem();

            if (selectedItem == activeConnectionConfigDTO) {
                return;
            }

            if (!dragging && checkDirty()) {
                showConnection(selectedItem);
                connectionsListView.getSelectionModel().select(activeConnectionConfigDTO);
            } else {
                connectionsListView.getSelectionModel().select(activeConnectionConfigDTO);
            }
        });

        setOnDragDetected(cell);
        setOnDragOver(cell);
        setOnDragEntered(cell);
        setOnDragDropped(cell);
        cell.setOnDragDone(DragEvent::consume);

        return cell;
    }

    private void setOnDragDetected(ConnectionCell cell) {
        cell.setOnDragDetected(event -> {
            if (cell.getItem() == null) {
                return;
            }

            if (!checkDirty()) {
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
            saveConnection();
            setUpDownLabel();
        });
    }

    private void loadConnectionListFromBackground() {
        ObservableList<ConnectionPropertiesDTO> list = FXCollections.observableArrayList(ConnectionPropertiesDTO.extractor());
        ConnectionHolder.getInstance().getSortedConnections()
                .forEach(c -> list.add(ConnectionTransformer.dtoToProps(c)));
        connectionsListView.setItems(list);
        executeOnLoadSettingsExtensions();
        LOGGER.debug("Loading connection list from background");
    }

    private void executeOnLoadSettingsExtensions() {
        connectionsListView.getItems().forEach(c -> {
            decodeLwtPayload(c);
            LwtConnectionExtensionDTO lwtDTO = new LwtConnectionExtensionDTO(c);
            for (LwtSettingsHook p : PluginManager.getInstance().getExtensions(LwtSettingsHook.class)) {
                lwtDTO = p.onLoadConnection(lwtDTO);
            }

            ConnectionTransformer.mergeProps(lwtDTO, c);
        });
    }

    private void decodeLwtPayload(ConnectionPropertiesDTO connectionPropertiesDTO) {
        String lwtPayload = connectionPropertiesDTO.getLwtPayload();
        if (lwtPayload != null) {
            connectionPropertiesDTO.getLwtPayloadProperty().set(new String(Base64.getDecoder().decode(lwtPayload)));
        }
    }

    private boolean checkDirty() {
        if (activeConnectionConfigDTO != null && activeConnectionConfigDTO.isDirty()) {
            if (confirmUnsavedConnectionSync()) {
                return handleConfirmedConnectionSync();
            } else {
                return handleUnconfirmedConnectionSync();
            }
        }
        return true;
    }

    private boolean handleUnconfirmedConnectionSync() {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Discarding unsaved changes: {}", activeConnectionConfigDTO.getId());
        }

        if (activeConnectionConfigDTO.isUnpersisted()) {
            connectionsListView.getItems().remove(activeConnectionConfigDTO);

            if (LOGGER.isDebugEnabled()) {
                LOGGER.debug("Removing connection: {}", activeConnectionConfigDTO.getId());
            }
        } else {
            activeConnectionConfigDTO.getDirtyProperty().set(false);
            showConnection(activeConnectionConfigDTO);

            if (LOGGER.isDebugEnabled()) {
                LOGGER.debug("Dirty Property set to false: {}", activeConnectionConfigDTO.getId());
            }
        }
        return true;
    }

    private boolean handleConfirmedConnectionSync() {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Unsaved connection status confirmed: {}", activeConnectionConfigDTO.getId());
        }
        return saveConnection();
    }

    @FXML
    public void onAddClicked() {
        LOGGER.debug("Add new connection clicked");

        if (checkDirty()) {
            addConnection();
        }
    }

    @FXML
    public void onRemoveClicked() {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Remove connection clicked: {}", activeConnectionConfigDTO.getId());
        }

        if (checkDirty()) {
            dropConnection();
        }
    }

    @FXML
    public void onCancelClicked() {
        ConnectionPropertiesDTO config = connectionsListView.getSelectionModel().getSelectedItem();
        logCancelClick(config);
        handleCancelClick(config);
    }

    private void handleCancelClick(ConnectionPropertiesDTO config) {
        if (config != null && config.isDirty()) {

            if (confirmUnsavedConnectionSync()) {
                if (LOGGER.isDebugEnabled()) {
                    LOGGER.debug("Unsaved connection status confirmed: {}", activeConnectionConfigDTO.getId());
                }

                if (!saveConnection()) {
                    return;
                }
            } else {
                if (LOGGER.isDebugEnabled()) {
                    LOGGER.debug("Discarding unsaved changes: {}", activeConnectionConfigDTO.getId());
                }

                setDirty(false);
            }
        }
        closeDialog();
    }

    private void logCancelClick(ConnectionPropertiesDTO config) {
        if (LOGGER.isDebugEnabled()) {
            if (config != null) {
                LOGGER.debug("Cancel editing clicked: {}", activeConnectionConfigDTO.getId());
            } else {
                LOGGER.debug("Cancel editing clicked without selected connection");
            }
        }
    }

    @FXML
    public void onApplyClicked() {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Apply changes clicked: {}", activeConnectionConfigDTO.getId());
        }
        if (saveConnection()) {
            showConnection(activeConnectionConfigDTO);
        }
    }

    @FXML
    public void onSaveClicked() {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Save changes clicked: {}", activeConnectionConfigDTO.getId());
        }
        if (saveConnection()) {
            closeDialog();
        }
    }

    private boolean doChecks() {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Do form checks for connection: {}", activeConnectionConfigDTO.getId());
        }

        boolean checksPassed = !checkName(nameTextField, true);
        checksPassed |= !checkUrl(urlTextField, true);
        checksPassed |= !checkPort(portTextField, true);
        checksPassed |= !checkClientID(clientIdTextField, true);
        if (lwtComboBox.getSelectionModel().getSelectedItem().equals(Lwt.ON)) {
            checksPassed |= !CheckTopicHelper.checkPublishTopic(lwtTopicComboBox, true);
        }

        return !checksPassed;
    }

    private void closeDialog() {
        Stage stage = (Stage) saveButton.getScene().getWindow();
        stage.close();
    }

    private void keyHandling(KeyEvent event) {
        if (KeyCode.ESCAPE == event.getCode()) {
            closeDialog();
        }
    }

    private void setUpDownLabel() {
        if (connectionsListView.getSelectionModel().getSelectedIndex() == 0) {
            upLabel.setDisable(true);
        }

        if (connectionsListView.getSelectionModel().getSelectedIndex() == connectionsListView.getItems().size() - 1) {
            downLabel.setDisable(true);
        }
    }

    private void showConnection(ConnectionPropertiesDTO config) {

        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Show connection: {}", config.getId());
        }

        dirtyCheckEnabled.set(false);

        activeConnectionConfigDTO = config;
        connectionConfigTabPane.setDisable(false);
        saveButton.setDisable(false);
        applyButton.setDisable(false);
        dropLabel.setDisable(false);
        upLabel.setDisable(false);
        downLabel.setDisable(false);

        setUpDownLabel();

        if (activeConnectionConfigDTO != null) {

            executeOnShowConnectionExtensions();

            nameTextField.setText(activeConnectionConfigDTO.getName());
            urlTextField.setText(activeConnectionConfigDTO.getUrl());
            portTextField.setText(Integer.toString(activeConnectionConfigDTO.getPort()));
            clientIdTextField.setText(activeConnectionConfigDTO.getClientId());
            usernameTextField.setText(activeConnectionConfigDTO.getUsername());
            passwordField.setText(activeConnectionConfigDTO.getPassword());
            cleanSessionCheckBox.setSelected(activeConnectionConfigDTO.isCleanSession());
            mqttVersionComboBox.getSelectionModel().select(activeConnectionConfigDTO.getMqttVersion());
            tlsSslGridPane.setDisable(activeConnectionConfigDTO.getSslProperty().getValue().equals(TlsSsl.OFF));
            tlsComboBox.getSelectionModel().select(activeConnectionConfigDTO.getSsl());
            sslKeystoreTextField.setText(activeConnectionConfigDTO.getSslKeystore());
            sslKeystorePasswordTextField.setText(activeConnectionConfigDTO.getSslKeystorePassword());
            proxyGridPane.setDisable(activeConnectionConfigDTO.getProxyProperty().getValue().equals(Proxy.OFF));
            proxyComboBox.getSelectionModel().select(activeConnectionConfigDTO.getProxy());
            sshHostTextField.setText(activeConnectionConfigDTO.getSshHost());
            sshPortTextField.setText(Integer.toString(activeConnectionConfigDTO.getSshPort()));
            localPortTextField.setText(Integer.toString(activeConnectionConfigDTO.getLocalPort()));

            if (activeConnectionConfigDTO.getAuth().equals(Auth.OFF)) {
                authUsernameTextField.setDisable(true);
                authPasswordField.setDisable(true);
                authKeyfileHBox.setDisable(true);
            } else if (activeConnectionConfigDTO.getAuth().equals(Auth.PASSWORD)) {
                authUsernameTextField.setDisable(false);
                authPasswordField.setDisable(false);
                authKeyfileHBox.setDisable(true);
            } else if (activeConnectionConfigDTO.getAuth().equals(Auth.KEYFILE)) {
                authUsernameTextField.setDisable(false);
                authPasswordField.setDisable(true);
                authKeyfileHBox.setDisable(false);
            }

            authComboBox.getSelectionModel().select(activeConnectionConfigDTO.getAuth());
            authUsernameTextField.setText(activeConnectionConfigDTO.getAuthUsername());
            authPasswordField.setText(activeConnectionConfigDTO.getAuthPassword());
            authKeyFileTextField.setText(activeConnectionConfigDTO.getAuthKeyfile());
            lwtContentVBox.setDisable(activeConnectionConfigDTO.getLwt().equals(Lwt.OFF));
            lwtComboBox.getSelectionModel().select(activeConnectionConfigDTO.getLwt());
            lwtTopicComboBox.getEditor().setText(activeConnectionConfigDTO.getLwtTopic());
            lwtQoSComboBox.getSelectionModel().select(activeConnectionConfigDTO.getLwtQos());
            lwtRetainedCheckBox.setSelected(activeConnectionConfigDTO.isLwtRetained());
            if (activeConnectionConfigDTO.getLwtPayload() != null) {
                lwtPayloadCodeArea.replaceText(activeConnectionConfigDTO.getLwtPayload());
            }

            internalIdLabel.setText(resources.getString("connectionSettingsViewInternalIdLabel") + ": " + activeConnectionConfigDTO.getId());
            activeConnectionConfigDTO.getDirtyProperty().set(false);
            applyButton.setDisable(true);
            saveButton.setDisable(true);
        }

        dirtyCheckEnabled.set(true);
    }

    private void executeOnShowConnectionExtensions() {
        LwtConnectionExtensionDTO lwtConnectionExtensionDTO = new LwtConnectionExtensionDTO(activeConnectionConfigDTO);
        for (LwtSettingsHook p : PluginManager.getInstance().getExtensions(LwtSettingsHook.class)) {
            lwtConnectionExtensionDTO = p.onShowConnection(lwtConnectionExtensionDTO);
        }
        ConnectionTransformer.mergeProps(lwtConnectionExtensionDTO, activeConnectionConfigDTO);
    }

    @Override
    public void setDirty(boolean dirty) {
        if (dirtyCheckEnabled.get()) {
            activeConnectionConfigDTO.getDirtyProperty().set(dirty);
            applyButton.setDisable(!dirty);
            saveButton.setDisable(!dirty);
            upLabel.setDisable(dirty);
            downLabel.setDisable(dirty);
        }
    }

    @FXML
    public void onGenerateClientIdClick() {
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Generating Client ID for connection clicked: {}", activeConnectionConfigDTO.getId());
        }
        clientIdTextField.setText(UUID.randomUUID().toString());
    }

    private void addConnection() {

        ConnectionPropertiesDTO newConfig = ConnectionTransformer.dtoToProps(ConnectionConfigDTO.builder()
                .id(UUID.randomUUID().toString())
                .name(resources.getString("connectionSettingsViewControllerNewConnectionName"))
                .build());

        newConfig.getUnpersistedProperty().set(true);
        newConfig.getDirtyProperty().set(true);

        connectionsListView.getItems().add(newConfig);
        connectionsListView.getSelectionModel().select(newConfig);
        newConfig.getUnpersistedProperty().set(true);
        showConnection(newConfig);
        setDirty(true);

        nameTextField.requestFocus();
    }

    private void dropConnection() {

        ConnectionPropertiesDTO selectedItem = connectionsListView.getSelectionModel().getSelectedItem();

        boolean confirmed = AlertHelper.confirm(
                resources.getString("connectionSettingsViewControllerDeleteTitle"),
                resources.getString("connectionSettingsViewControllerDeleteHeader") + "? (" + selectedItem + ")",
                null,
                resources.getString("commonNoButton"),
                resources.getString("commonYesButton")
        );

        int selectedIndex = connectionsListView.getSelectionModel().getSelectedIndex();

        if (confirmed) {

            LOGGER.info("Disconnect connection selected");

            CorreoMqttClient client = ConnectionHolder.getInstance().getClient(selectedItem.getId());
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
                    MessageTaskFactory.disconnect(selectedItem.getId(), false);

                    waitForDisconnectIds.put(selectedItem.getId(), selectedIndex);
                }
            } else {
                removeConnectionAndSave(selectedItem, selectedIndex);
            }
        }
    }

    private void removeConnectionAndSave(ConnectionPropertiesDTO selectedItem, int selectedIndex) {
        connectionConfigTabPane.setDisable(true);

        connectionsListView.getItems().remove(selectedItem);

        //TODO ensure that the passwords are also removed

        List<ConnectionConfigDTO> connections = ConnectionTransformer.propsListToDtoList(connectionsListView.getItems());
        KeyringHandler.getInstance().retryWithMasterPassword(
                masterPassword -> SettingsProvider.getInstance().saveConnections(connections, masterPassword),
                resources.getString("onPasswordSaveFailedTitle"),
                resources.getString("onPasswordSaveFailedHeader"),
                resources.getString("onPasswordSaveFailedContent"),
                resources.getString("onPasswordSaveFailedGiveUp"),
                resources.getString("onPasswordSaveFailedTryAgain")
        );

        if (selectedIndex != 0) {
            selectedIndex--;
            if (selectedIndex < 0) {
                selectedIndex = 0;
            }
        }

        if (connectionsListView.getItems().size() > selectedIndex) {
            connectionsListView.getSelectionModel().select(selectedIndex);
            showConnection(connectionsListView.getItems().get(selectedIndex));
        } else {
            clearConnectionsForm();
            applyButton.setDisable(true);
            saveButton.setDisable(true);
            dropLabel.setDisable(true);
            upLabel.setDisable(true);
            downLabel.setDisable(true);
        }
    }

    private boolean saveConnection() {

        if (doChecks()) {

            activeConnectionConfigDTO.getNameProperty().set(nameTextField.getText());
            activeConnectionConfigDTO.getUrlProperty().set(urlTextField.getText());
            activeConnectionConfigDTO.getPortProperty().set(Integer.parseInt(portTextField.getText()));
            activeConnectionConfigDTO.getClientIdProperty().set(clientIdTextField.getText());
            activeConnectionConfigDTO.getUsernameProperty().set(usernameTextField.getText());
            activeConnectionConfigDTO.getPasswordProperty().set(passwordField.getText());
            activeConnectionConfigDTO.getCleanSessionProperty().set(cleanSessionCheckBox.isSelected());
            activeConnectionConfigDTO.getMqttVersionProperty().setValue(mqttVersionComboBox.getSelectionModel().getSelectedItem());
            activeConnectionConfigDTO.getSslProperty().setValue(tlsComboBox.getSelectionModel().getSelectedItem());
            activeConnectionConfigDTO.getSslKeystoreProperty().set(sslKeystoreTextField.getText());
            activeConnectionConfigDTO.getSslKeystorePasswordProperty().set(sslKeystorePasswordTextField.getText());
            activeConnectionConfigDTO.getProxyProperty().setValue(proxyComboBox.getSelectionModel().getSelectedItem());
            activeConnectionConfigDTO.getSshHostProperty().set(sshHostTextField.getText());
            activeConnectionConfigDTO.getSshPortProperty().set(Integer.parseInt(sshPortTextField.getText()));
            activeConnectionConfigDTO.getLocalPortProperty().set(Integer.parseInt(localPortTextField.getText()));
            if (authComboBox.getSelectionModel().getSelectedItem().equals(Auth.PASSWORD)) {
                activeConnectionConfigDTO.getAuthProperty().setValue(Auth.PASSWORD);
            } else if (authComboBox.getSelectionModel().getSelectedItem().equals(Auth.KEYFILE)) {
                activeConnectionConfigDTO.getAuthProperty().setValue(Auth.KEYFILE);
            } else {
                activeConnectionConfigDTO.getAuthProperty().setValue(Auth.OFF);
            }
            activeConnectionConfigDTO.getAuthUsernameProperty().set(authUsernameTextField.getText());
            activeConnectionConfigDTO.getAuthPasswordProperty().set(authPasswordField.getText());
            activeConnectionConfigDTO.getAuthKeyfileProperty().set(authKeyFileTextField.getText());
            activeConnectionConfigDTO.getLwtProperty().setValue(lwtComboBox.getSelectionModel().getSelectedItem());
            activeConnectionConfigDTO.getLwtTopicProperty().set(lwtTopicComboBox.getEditor().getText());
            activeConnectionConfigDTO.getLwtQoSProperty().setValue(lwtQoSComboBox.getSelectionModel().getSelectedItem());
            activeConnectionConfigDTO.getLwtRetainedProperty().set(lwtRetainedCheckBox.isSelected());
            activeConnectionConfigDTO.getLwtPayloadProperty().set(lwtPayloadCodeArea.getText());

            activeConnectionConfigDTO = executeOnSaveSettingsExtensions(activeConnectionConfigDTO);
            executeOnUnloadSettingsExtensions();

            List<ConnectionConfigDTO> connections = ConnectionTransformer.propsListToDtoList(connectionsListView.getItems());
            KeyringHandler.getInstance().retryWithMasterPassword(
                    masterPassword -> SettingsProvider.getInstance().saveConnections(connections, masterPassword),
                    resources.getString("onPasswordSaveFailedTitle"),
                    resources.getString("onPasswordSaveFailedHeader"),
                    resources.getString("onPasswordSaveFailedContent"),
                    resources.getString("onPasswordSaveFailedGiveUp"),
                    resources.getString("onPasswordSaveFailedTryAgain")
            );

            saveButton.setDisable(false);
            applyButton.setDisable(false);
            dropLabel.setDisable(false);
            upLabel.setDisable(false);
            downLabel.setDisable(false);

            activeConnectionConfigDTO.getDirtyProperty().set(false);
            activeConnectionConfigDTO.getUnpersistedProperty().set(false);

            executeOnLoadSettingsExtensions();
            return true;
        }

        return false;
    }

    private ConnectionPropertiesDTO executeOnSaveSettingsExtensions(ConnectionPropertiesDTO activeConnectionConfigDTO) {
        LwtConnectionExtensionDTO lwtDTO = new LwtConnectionExtensionDTO(activeConnectionConfigDTO);
        for (LwtSettingsHook p : PluginManager.getInstance().getExtensions(LwtSettingsHook.class)) {
            lwtDTO = p.onSaveConnection(lwtDTO);
        }
        return ConnectionTransformer.mergeProps(lwtDTO, activeConnectionConfigDTO);
    }

    private void executeOnUnloadSettingsExtensions() {
        connectionsListView.getItems().forEach(c -> {
            LwtConnectionExtensionDTO lwtDTO = new LwtConnectionExtensionDTO(c);
            for (LwtSettingsHook p : PluginManager.getInstance().getExtensions(LwtSettingsHook.class)) {
                lwtDTO = p.onUnloadConnection(lwtDTO);
            }
            ConnectionTransformer.mergeProps(lwtDTO, c);
            encodeLwtPayload(c);
        });
    }

    private void encodeLwtPayload(ConnectionPropertiesDTO c) {
        String lwtPayload = c.getLwtPayload();
        if (lwtPayload != null) {
            c.getLwtPayloadProperty().set(Base64.getEncoder().encodeToString(lwtPayload.getBytes()));
        }
    }

    private boolean checkName(TextField textField, boolean save) {
        if (!checkRequired(textField)) {
            setError(textField, save, resources.getString("validationNameIsEmpty"));
            return false;
        }

        //check name collision
        for (ConnectionPropertiesDTO connectionConfigDTO : connectionsListView.getItems()) {
            if (connectionConfigDTO == activeConnectionConfigDTO) { // I do not want to check myself.
                continue;
            }
            if (connectionConfigDTO.getName().equals(nameTextField.getText())) {
                setError(textField, save, resources.getString("validationNameAlreadyUsed"));
                return false;
            }
        }

        if (textField.getText().length() > 32) {
            setError(textField, save, resources.getString("validationNameIsTooLong"));
            return false;
        }

        nameTextField.getStyleClass().clear();
        nameTextField.getStyleClass().addAll(TEXT_FIELD, TEXT_INPUT);
        return true;
    }

    private boolean checkUrl(TextField textField, boolean save) {
        if (!checkRequired(textField)) {
            setError(textField, save, resources.getString("validationConnectionIsEmpty"));
            return false;
        }

        urlTextField.getStyleClass().clear();
        urlTextField.getStyleClass().addAll(TEXT_FIELD, TEXT_INPUT);
        return true;
    }

    private boolean checkPort(TextField textField, boolean save) {
        if (!checkRequired(textField)) {
            setError(textField, save, resources.getString("validationPortIsEmpty"));
            return false;
        } else if (!textField.getText().matches("-?\\d+") || textField.getText().length() > 5) {
            setError(textField, save, resources.getString("validationInvalidPort"));
            return false;
        }

        portTextField.getStyleClass().clear();
        portTextField.getStyleClass().addAll(TEXT_FIELD, TEXT_INPUT);
        return true;
    }

    private boolean checkClientID(TextField textField, boolean save) {
        if (!checkRequired(textField)) {
            setError(textField, save, resources.getString("validationClientIdIsEmpty"));
            return false;
        } else if (textField.getText().length() > CLIENT_ID_MAX_SIZE) {
            setError(textField, save, resources.getString("validationClientIdIsTooLong"));
            return false;
        }

        clientIdTextField.getStyleClass().clear();
        clientIdTextField.getStyleClass().addAll(TEXT_FIELD, TEXT_INPUT);
        return true;
    }

    private void setError(TextField textField, boolean save, String tooltipText) {
        if (save) {
            textField.getStyleClass().add("errorOnSave");
        }

        textField.setTooltip(new Tooltip(tooltipText));
        textField.getStyleClass().add(EXCLAMATION_CIRCLE_SOLID);
    }

    private boolean checkRequired(TextField textField) {
        return !(textField.getText() == null || textField.getText().isEmpty());
    }

    private void clearConnectionsForm() {
        LOGGER.debug("Clearing connections form");
        nameTextField.clear();
        urlTextField.clear();
        portTextField.setText(null);
        clientIdTextField.clear();
        usernameTextField.clear();
        passwordField.clear();
        sslKeystoreTextField.clear();
        sslKeystorePasswordTextField.clear();
        internalIdLabel.setText("");
        nameTextField.getStyleClass().removeAll(EMPTY_ERROR_CLASS);
        nameTextField.getStyleClass().removeAll(EXCLAMATION_CIRCLE_SOLID);
        urlTextField.getStyleClass().removeAll(EMPTY_ERROR_CLASS);
        urlTextField.getStyleClass().removeAll(EXCLAMATION_CIRCLE_SOLID);
        portTextField.getStyleClass().removeAll(EMPTY_ERROR_CLASS);
        portTextField.getStyleClass().removeAll(EXCLAMATION_CIRCLE_SOLID);
        clientIdTextField.getStyleClass().removeAll(EMPTY_ERROR_CLASS);
        clientIdTextField.getStyleClass().removeAll(EXCLAMATION_CIRCLE_SOLID);
    }

    private boolean confirmUnsavedConnectionSync() {
        return AlertHelper.confirm(
                resources.getString("connectionSettingsViewControllerUnsavedTitle"),
                resources.getString("connectionSettingsViewControllerUnsavedHeader"),
                resources.getString("connectionSettingsViewControllerUnsavedContent"),
                resources.getString("commonSaveButton"),
                resources.getString("commonDiscardButton")
        );
    }

    @FXML
    private void selectKeystore() {
        FileChooser fileChooser = new FileChooser();
        File selectedFile = fileChooser.showOpenDialog(containerAnchorPane.getScene().getWindow());
        sslKeystoreTextField.setText(selectedFile.toString());
    }

    @FXML
    private void selectKeyfile() {
        FileChooser fileChooser = new FileChooser();
        File selectedFile = fileChooser.showOpenDialog(containerAnchorPane.getScene().getWindow());
        authKeyFileTextField.setText(selectedFile.toString());
    }

    @FXML
    private void moveConnectionUp() {
        int currentIndex = connectionsListView.getSelectionModel().getSelectedIndex();
        ConnectionPropertiesDTO previous = connectionsListView.getItems().get(currentIndex - 1);
        ConnectionPropertiesDTO current = connectionsListView.getItems().get(currentIndex);
        connectionsListView.getItems().set(currentIndex - 1, current);
        connectionsListView.getItems().set(currentIndex, previous);
        connectionsListView.getSelectionModel().select(current);
        saveConnection();
        showConnection(current);
    }

    @FXML
    private void moveConnectionDown() {
        int currentIndex = connectionsListView.getSelectionModel().getSelectedIndex();
        ConnectionPropertiesDTO next = connectionsListView.getItems().get(currentIndex + 1);
        ConnectionPropertiesDTO current = connectionsListView.getItems().get(currentIndex);
        connectionsListView.getItems().set(currentIndex + 1, current);
        connectionsListView.getItems().set(currentIndex, next);
        connectionsListView.getSelectionModel().select(current);
        saveConnection();
        showConnection(current);
    }

    public void openExport(boolean autoNew) {
        ConnectionExportViewController.showAsDialog(connectionExportViewDelegate);
        if (autoNew) {
            //result.getController().onAddClicked(); TODO
            LOGGER.debug("Open settings with new default connection");
        } else {
            LOGGER.debug("Open settings for existing connections");
        }
    }

    @FXML
    public void openExport() {
        openExport(false);


    }

    public void openImport(boolean autoNew) {
        Stage stage = (Stage) containerAnchorPane.getScene().getWindow();

        FileChooser fileChooser = new FileChooser();
        fileChooser.setTitle(resources.getString("importUtilsTitle"));
        FileChooser.ExtensionFilter extFilter = new FileChooser.ExtensionFilter(resources.getString("importUtilsDescription"), "*.json");
        fileChooser.getExtensionFilters().add(extFilter);

        File file = fileChooser.showOpenDialog(stage);
        if (file != null) {
            MessageTaskFactory.importConnection(file);
            ConnectionImportViewController.showAsDialog(connectionImportViewDelegate);
        }
    }

    @FXML
    public void openImport() {
        openImport(false);

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

    @Override
    public void onDisconnectFromConnectionDeleted(String connectionId) {
        if (waitForDisconnectIds.get(connectionId) != null) {
            delegate.closeTab(connectionsListView.getItems().get(waitForDisconnectIds.get(connectionId)).getName());
            removeConnectionAndSave(connectionsListView.getItems().get(waitForDisconnectIds.get(connectionId)),
                    waitForDisconnectIds.get(connectionId));
            waitForDisconnectIds.remove(connectionId);
        }
    }

    @Override
    public void onConnect() {
        // do nothing
    }

    @Override
    public void onConnectRunning() {
        // do nothing
    }

    @Override
    public void onConnectionFailed(Throwable message) {
        // do nothing
    }

    @Override
    public void onConnectionLost() {
        // do nothing
    }

    @Override
    public void onDisconnect() {
        // do nothing
    }

    @Override
    public void onDisconnectFailed(Throwable exception) {
        // do nothing
    }

    @Override
    public void onDisconnectRunning() {
        // do nothing
    }

    @Override
    public void onConnectionReconnected() {
        // do nothing
    }

    @Override
    public void onReconnectFailed(AtomicInteger triedReconnects, int maxReconnects) {
        // do nothing
    }

    @Override
    public void onCleanUp(String connectinId) {
        ConfigDispatcher.getInstance().removeObserver(this);
        ConnectionLifecycleDispatcher.getInstance().removeObserver(this);
        ImportConnectionDispatcher.getInstance().removeObserver(this);
    }

    @Override
    public String getConnectionId() {
        return null;
    }


    @Override
    public void onImportSucceeded(ConnectionExportDTO connectionExportDTO) {
        // TODO Triggered twice, fix to only trigger once after import is completed
        loadConnectionListFromBackground();
    }


    @Override
    public void onImportCancelled(File file) {

    }

    @Override
    public void onImportFailed(File file, Throwable exception) {

    }
}
