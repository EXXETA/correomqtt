package com.exxeta.correomqtt.gui.controller;

import com.exxeta.correomqtt.business.dispatcher.ConfigDispatcher;
import com.exxeta.correomqtt.business.dispatcher.ConfigObserver;
import com.exxeta.correomqtt.business.dispatcher.ConnectionLifecycleDispatcher;
import com.exxeta.correomqtt.business.dispatcher.ConnectionLifecycleObserver;
import com.exxeta.correomqtt.business.model.Auth;
import com.exxeta.correomqtt.business.model.ConnectionConfigDTO;
import com.exxeta.correomqtt.business.model.CorreoMqttVersion;
import com.exxeta.correomqtt.business.model.Lwt;
import com.exxeta.correomqtt.business.model.Proxy;
import com.exxeta.correomqtt.business.model.Qos;
import com.exxeta.correomqtt.business.model.TlsSsl;
import com.exxeta.correomqtt.business.mqtt.CorreoMqttClient;
import com.exxeta.correomqtt.business.services.ConfigService;
import com.exxeta.correomqtt.business.utils.ConnectionHolder;
import com.exxeta.correomqtt.gui.business.TaskFactory;
import com.exxeta.correomqtt.gui.cell.ConnectionCell;
import com.exxeta.correomqtt.gui.cell.GenericCell;
import com.exxeta.correomqtt.gui.helper.CheckTopicHelper;
import com.exxeta.correomqtt.gui.model.ConnectionPropertiesDTO;
import com.exxeta.correomqtt.gui.model.GenericCellModel;
import com.exxeta.correomqtt.gui.model.WindowProperty;
import com.exxeta.correomqtt.gui.model.WindowType;
import com.exxeta.correomqtt.gui.transformer.ConnectionTransformer;
import com.exxeta.correomqtt.gui.utils.WindowHelper;
import javafx.collections.FXCollections;
import javafx.collections.ObservableList;
import javafx.fxml.FXML;
import javafx.scene.control.Alert;
import javafx.scene.control.Button;
import javafx.scene.control.ButtonType;
import javafx.scene.control.CheckBox;
import javafx.scene.control.ComboBox;
import javafx.scene.control.DialogPane;
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
import org.fxmisc.flowless.VirtualizedScrollPane;
import org.fxmisc.richtext.CodeArea;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.File;
import java.util.Base64;
import java.util.HashMap;
import java.util.Map;
import java.util.Optional;
import java.util.ResourceBundle;
import java.util.UUID;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicInteger;

public class ConnectionSettingsViewController extends BaseController implements ConfigObserver, ConnectionLifecycleObserver {

    private static final String TEXT_FIELD = "text-field";
    private static final String TEXT_INPUT = "text-input";
    private static final Logger LOGGER = LoggerFactory.getLogger(ConnectionSettingsViewController.class);
    private static final int CLIENT_ID_MAX_SIZE = 64;
    private final ConnectionSettingsViewDelegate delegate;

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
    private CheckBox lwtMessageIdCheckBox;
    @FXML
    private CheckBox lwtAnswerExpectedCheckBox;
    @FXML
    private CheckBox lwtRetainedCheckBox;
    @FXML
    private CodeArea lwtPayloadCodeArea;
    @FXML
    private Pane lwtPayloadPane;

    private static ResourceBundle resources;

    private ConnectionPropertiesDTO activeConnectionConfigDTO;

    private AtomicBoolean dirtyCheckEnabled = new AtomicBoolean(true);
    private boolean dragging;
    Map<String, Integer> waitForDisconnectIds = new HashMap<>();

    public ConnectionSettingsViewController(ConnectionSettingsViewDelegate delegate) {
        this.delegate = delegate;
        ConfigDispatcher.getInstance().addObserver(this);
        ConnectionLifecycleDispatcher.getInstance().addObserver(this);
    }

    public static LoaderResult<ConnectionSettingsViewController> load(ConnectionSettingsViewDelegate delegate) {
        return load(ConnectionSettingsViewController.class, "connectionSettingsView.fxml",
                () -> new ConnectionSettingsViewController(delegate));
    }

    public static void showAsDialog(ConnectionSettingsViewDelegate delegate) {


        Map<Object, Object> properties = new HashMap<>();
        properties.put(WindowProperty.WINDOW_TYPE, WindowType.CONNECTION_SETTINGS);

        if (WindowHelper.focusWindowIfAlreadyThere(properties)) {
            return;
        }
        LoaderResult<ConnectionSettingsViewController> result = load(delegate);
        resources = result.getResourceBundle();

        showAsDialog(result, resources.getString("connectionSettingsViewControllerTitle"), properties, false, false, null,
                event -> result.getController().keyHandling(event));
    }

    @FXML
    public void initialize() {

        containerAnchorPane.getStyleClass().add(ConfigService.getInstance().getThemeSettings().getActiveTheme().getIconMode());


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

        if (!ConfigService.getInstance().getSettings().isExtraFeatures()) {
            lwtMessageIdCheckBox.setVisible(false);
            lwtAnswerExpectedCheckBox.setVisible(false);
        }

        connectionsListView.setCellFactory(this::createCell);

        saveButton.setDisable(true);
        applyButton.setDisable(true);
        dropLabel.setDisable(true);
        upLabel.setDisable(true);
        downLabel.setDisable(true);

        nameTextField.lengthProperty().addListener((observabyyle, oldValue, newValue) ->
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
        lwtMessageIdCheckBox.selectedProperty().addListener((observable, oldValue, newValue) -> setDirty(true));
        lwtAnswerExpectedCheckBox.selectedProperty().addListener((observable, oldValue, newValue) -> setDirty(true));
        lwtRetainedCheckBox.selectedProperty().addListener((observable, oldValue, newValue) -> setDirty(true));
        lwtPayloadCodeArea.textProperty().addListener(((observable, oldValue, newValue) -> setDirty(true)));

        internalIdLabel.setText("");
    }

    private <T extends GenericCellModel> StringConverter<T> getStringConverter(){
        return new StringConverter<T>() {
            @Override
            public String toString(T object) {
                if (object == null) {
                    return null;
                }
                String translationKey = object.getLabelTranslationKey();
                if(translationKey != null) {
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

    public void setCheckBoxes() {
        if (lwtMessageIdCheckBox.isSelected()) {
            lwtAnswerExpectedCheckBox.setDisable(false);
        } else {
            lwtAnswerExpectedCheckBox.setDisable(true);
            lwtAnswerExpectedCheckBox.setSelected(false);
        }
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
                    int draggedIdx = connectionsListView.getSelectionModel().getSelectedIndex();
                    int thisIdx = items.indexOf(cell.getItem());

                    if (draggedIdx > thisIdx && draggedIdx > -1 && thisIdx > -1) {
                        for (int i = draggedIdx; i > thisIdx; i--) {
                            ConnectionPropertiesDTO temp = connectionsListView.getItems().get(i);
                            connectionsListView.getItems().set(i, connectionsListView.getItems().get(i - 1));
                            connectionsListView.getItems().set(i - 1, temp);
                        }
                    }

                    if (draggedIdx < thisIdx && draggedIdx > -1 && thisIdx > -1) {
                        for (int i = draggedIdx; i < thisIdx; i++) {
                            ConnectionPropertiesDTO temp = connectionsListView.getItems().get(i);
                            connectionsListView.getItems().set(i, connectionsListView.getItems().get(i + 1));
                            connectionsListView.getItems().set(i + 1, temp);
                        }
                    }

                    connectionsListView.getSelectionModel().select(thisIdx);
                }
            }
        });
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
        LOGGER.debug("Loading connection list from background");
    }

    private boolean checkDirty() {
        if (activeConnectionConfigDTO != null && activeConnectionConfigDTO.isDirty()) {
            if (confirmUnsavedConnectionSync()) {
                if (LOGGER.isDebugEnabled()) {
                    LOGGER.debug("Unsaved connection status confirmed: {}", activeConnectionConfigDTO.getId());
                }
                return saveConnection();
            } else {
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
            }
        }
        return true;
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
        if (config != null && LOGGER.isDebugEnabled()) {
            LOGGER.debug("Cancel editing clicked: {}", activeConnectionConfigDTO.getId());
        } else if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Cancel editing clicked without selected connection");
        }

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
            lwtMessageIdCheckBox.setSelected(activeConnectionConfigDTO.getLwtMessageId());
            lwtAnswerExpectedCheckBox.setSelected(activeConnectionConfigDTO.getLwtAnswerExpected());
            lwtAnswerExpectedCheckBox.setDisable(!activeConnectionConfigDTO.getLwtMessageId());
            lwtRetainedCheckBox.setSelected(activeConnectionConfigDTO.getLwtRetained());
            if (activeConnectionConfigDTO.getLwtPayload() != null) {
                lwtPayloadCodeArea.replaceText(new String(Base64.getDecoder().decode(activeConnectionConfigDTO.getLwtPayload())));
            }

            internalIdLabel.setText(resources.getString("connectionSettingsViewInternalIdLabel") + ": " + activeConnectionConfigDTO.getId());
            activeConnectionConfigDTO.getDirtyProperty().set(false);
            applyButton.setDisable(true);
            saveButton.setDisable(true);
        }

        dirtyCheckEnabled.set(true);
    }

    private void setDirty(boolean dirty) {
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

        //TODO extract to helper
        Alert alert = new Alert(Alert.AlertType.CONFIRMATION);
        DialogPane dialogPane = alert.getDialogPane();
        String cssPath = ConfigService.getInstance().getCssPath();
        if (cssPath != null) {
            dialogPane.getStylesheets().add(cssPath);
        }
        alert.setTitle(resources.getString("connectionSettingsViewControllerDeleteTitle"));
        alert.setHeaderText(resources.getString("connectionSettingsViewControllerDeleteHeader") + "? (" + selectedItem + ")");

        ButtonType no = new ButtonType(resources.getString("commonNoButton"));
        ButtonType yes = new ButtonType(resources.getString("commonYesButton"));

        alert.getButtonTypes().setAll(yes, no);

        Optional<ButtonType> result = alert.showAndWait();

        if (!result.isPresent()) {
            LOGGER.error("No result from confirm drop connection dialog.");
            return;
        }

        int selectedIndex = connectionsListView.getSelectionModel().getSelectedIndex();

        if (result.get().equals(yes)) {

            LOGGER.info("Disconnect connection selected");

            CorreoMqttClient client = ConnectionHolder.getInstance().getClient(selectedItem.getId());
            if (client != null) {
                LOGGER.info("Connection is still connected");

                //TODO extract to helper
                Alert disconnectAlert = new Alert(Alert.AlertType.CONFIRMATION);
                DialogPane disconnectDialogPane = disconnectAlert.getDialogPane();
                if (cssPath != null) {
                    disconnectDialogPane.getStylesheets().add(cssPath);
                }
                disconnectAlert.setTitle(resources.getString("connectionSettingsViewControllerStillInUseTitle"));
                disconnectAlert.setHeaderText(resources.getString("connectionSettingsViewControllerStillInUseHeader"));

                disconnectAlert.getButtonTypes().setAll(yes, no);

                Optional<ButtonType> disconnectResult = disconnectAlert.showAndWait();

                if (!disconnectResult.isPresent()) {
                    LOGGER.info("No result from confirm disconnect connection dialog.");
                    return;
                }

                if (disconnectResult.get().equals(yes)) {
                    LOGGER.info("Disconnect");
                    TaskFactory.disconnect(selectedItem.getId());

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

        //TODO check result in background
        ConfigService.getInstance().saveConnections(
                ConnectionTransformer.propsListToDtoList(connectionsListView.getItems())
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
            activeConnectionConfigDTO.getLwtMessageIdProperty().set(lwtMessageIdCheckBox.isSelected());
            activeConnectionConfigDTO.getLwtAnswerExpectedProperty().set(lwtAnswerExpectedCheckBox.isSelected());
            activeConnectionConfigDTO.getLwtRetainedProperty().set(lwtRetainedCheckBox.isSelected());
            activeConnectionConfigDTO.getLwtPayloadProperty().set(Base64.getEncoder().encodeToString(lwtPayloadCodeArea.getText().getBytes()));

            if (ConfigService.getInstance().getSettings().isExtraFeatures() && lwtMessageIdCheckBox.isSelected()) {
                if (lwtAnswerExpectedCheckBox.isSelected()) {
                    activeConnectionConfigDTO.getLwtPayloadProperty().set("1" + activeConnectionConfigDTO.getLwtPayloadProperty());
                } else {
                    activeConnectionConfigDTO.getLwtPayloadProperty().set("0" + activeConnectionConfigDTO.getLwtPayloadProperty());
                }

                String uuid = UUID.randomUUID().toString();
                activeConnectionConfigDTO.getLwtPayloadProperty().set(uuid + activeConnectionConfigDTO.getLwtPayloadProperty());
            }


            ConfigService.getInstance().saveConnections(
                    ConnectionTransformer.propsListToDtoList(connectionsListView.getItems())
            );

            saveButton.setDisable(false);
            applyButton.setDisable(false);
            dropLabel.setDisable(false);
            upLabel.setDisable(false);
            downLabel.setDisable(false);

            activeConnectionConfigDTO.getDirtyProperty().set(false);
            activeConnectionConfigDTO.getUnpersistedProperty().set(false);

            return true;
        }

        return false;
    }

    private boolean checkName(TextField textField, boolean save) {
        if (!checkRequired(textField)) {
            setError(textField, save, resources.getString("validationNameIsEmpty"));
            return false;
        }

        //check name collision
        for (ConnectionPropertiesDTO ConnectionConfigDTO : connectionsListView.getItems()) {
            if (ConnectionConfigDTO == activeConnectionConfigDTO) { // I do not want to check myself.
                continue;
            }
            if (ConnectionConfigDTO.getName().equals(nameTextField.getText())) {
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
        textField.getStyleClass().add("exclamationCircleSolid");
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
        nameTextField.getStyleClass().removeAll("emptyError");
        nameTextField.getStyleClass().removeAll("exclamationCircleSolid");
        urlTextField.getStyleClass().removeAll("emptyError");
        urlTextField.getStyleClass().removeAll("exclamationCircleSolid");
        portTextField.getStyleClass().removeAll("emptyError");
        portTextField.getStyleClass().removeAll("exclamationCircleSolid");
        clientIdTextField.getStyleClass().removeAll("emptyError");
        clientIdTextField.getStyleClass().removeAll("exclamationCircleSolid");
    }

    private boolean confirmUnsavedConnectionSync() {
        Alert alert = new Alert(Alert.AlertType.CONFIRMATION);
        DialogPane dialogPane = alert.getDialogPane();
        String cssPath = ConfigService.getInstance().getCssPath();
        if (cssPath != null) {
            dialogPane.getStylesheets().add(cssPath);
        }
        alert.setTitle(resources.getString("connectionSettingsViewControllerUnsavedTitle"));
        alert.setHeaderText(resources.getString("connectionSettingsViewControllerUnsavedHeader"));
        alert.setContentText(resources.getString("connectionSettingsViewControllerUnsavedContent"));

        ButtonType save = new ButtonType(resources.getString("commonSaveButton"));
        ButtonType discard = new ButtonType(resources.getString("commonDiscardButton"));

        alert.getButtonTypes().setAll(save, discard);

        Optional<ButtonType> result = alert.showAndWait();
        if (!result.isPresent()) {
            LOGGER.error("No result from confirm unsaved connections dialog.");
            return false;
        }

        return result.get().equals(save);
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
    public void onSettingsUpdated() {
        if (!ConfigService.getInstance().getSettings().isExtraFeatures()) {
            lwtMessageIdCheckBox.setVisible(false);
            lwtMessageIdCheckBox.setSelected(false);
            lwtAnswerExpectedCheckBox.setVisible(false);
            lwtAnswerExpectedCheckBox.setSelected(false);
        } else {
            lwtMessageIdCheckBox.setVisible(true);
            lwtAnswerExpectedCheckBox.setVisible(true);
        }

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
    public String getConnectionId() {
        return null;
    }
}
