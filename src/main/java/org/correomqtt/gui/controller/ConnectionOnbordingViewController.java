package org.correomqtt.gui.controller;

import javafx.collections.FXCollections;
import javafx.event.ActionEvent;
import javafx.fxml.FXML;
import javafx.scene.control.Button;
import javafx.scene.control.ComboBox;
import javafx.scene.control.Label;
import javafx.scene.control.ListCell;
import javafx.scene.control.ListView;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseButton;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.HBox;
import javafx.scene.layout.VBox;
import org.correomqtt.business.dispatcher.ConfigDispatcher;
import org.correomqtt.business.dispatcher.ConfigObserver;
import org.correomqtt.business.utils.ConnectionHolder;
import org.correomqtt.gui.cell.ConnectionCell;
import org.correomqtt.gui.model.ConnectionPropertiesDTO;
import org.correomqtt.gui.transformer.ConnectionTransformer;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.Map;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.stream.Collectors;

public class ConnectionOnbordingViewController extends BaseController implements ConfigObserver {

    private static final Logger LOGGER = LoggerFactory.getLogger(ConnectionOnbordingViewController.class);
    @FXML
    public AnchorPane helloViewAnchor;
    @FXML
    public ComboBox<String> helloViewComboBox;
    @FXML
    public Button helloViewConnectButton;
    @FXML
    public Button editConnectionsButton;
    @FXML
    public HBox mainHBox;
    @FXML
    public Label noConnectionsLabel;
    @FXML
    public VBox helloViewVBox;
    @FXML
    public ListView<ConnectionPropertiesDTO> connectionListView;
    @FXML
    public HBox buttonBar;
    @FXML
    public HBox noConnectionsButtonBar;
    private ConnectionOnboardingDelegate connectionsOnboardingDelegate;
    private ConnectionSettingsViewDelegate connectionsSettingsViewDelegate;

    public ConnectionOnbordingViewController(ConnectionOnboardingDelegate connectionsOnboardingDelegate,
                                             ConnectionSettingsViewDelegate connectionSettingsViewDelegate) {
        super();
        this.connectionsOnboardingDelegate = connectionsOnboardingDelegate;
        this.connectionsSettingsViewDelegate = connectionSettingsViewDelegate;
        ConfigDispatcher.getInstance().addObserver(this);
    }

    public static LoaderResult<ConnectionOnbordingViewController> load(ConnectionOnboardingDelegate connectionsOnboardingDelegate,
                                                                       ConnectionSettingsViewDelegate connectionSettingsViewDelegate) {
        return load(ConnectionOnbordingViewController.class, "connectionOnboardingView.fxml",
                () -> new ConnectionOnbordingViewController(connectionsOnboardingDelegate, connectionSettingsViewDelegate));
    }

    public void setDelegate(ConnectionOnboardingDelegate delegate) {
        this.connectionsOnboardingDelegate = delegate;
    }

    @FXML
    public void initialize() {

        connectionListView.setCellFactory(this::createCell);

        connectionListView.setItems(FXCollections.observableArrayList(
                ConnectionTransformer.dtoListToPropList(ConnectionHolder.getInstance().getSortedConnections())
        ));

        updateConnections();

        connectionListView.addEventHandler(KeyEvent.KEY_PRESSED, event -> {
            if (KeyCode.ENTER == event.getCode()) {
                connect();
            }
        });

        connectionListView.requestFocus(); //ob tab add overrides this
    }

    private ListCell<ConnectionPropertiesDTO> createCell(ListView<ConnectionPropertiesDTO> connectionConfigDTOListView) {
        ConnectionCell cell = new ConnectionCell(connectionListView);
        cell.setOnMouseClicked(event -> onCellClicked(event, cell.getItem()));
        return cell;
    }

    private void onCellClicked(MouseEvent event, ConnectionPropertiesDTO configDTO) {
        if (configDTO != null && event.getButton().equals(MouseButton.PRIMARY) && event.getClickCount() == 2) {
            connect();
            if (LOGGER.isDebugEnabled()) {
                LOGGER.debug("Clicked on connect: {}", configDTO.getId());
            }
        }
    }

    private void updateConnections() {
        List<ConnectionPropertiesDTO> resultList = new ArrayList<>(connectionListView.getItems());

        List<ConnectionPropertiesDTO> newConnectionList = ConnectionTransformer.dtoListToPropList(ConnectionHolder.getInstance().getSortedConnections());

        AtomicInteger sortIndex = new AtomicInteger();
        Map<String, Integer> sorted = newConnectionList.stream()
                .collect(Collectors.toMap(ConnectionPropertiesDTO::getId, i -> sortIndex.getAndIncrement()));

        Map<String, ConnectionPropertiesDTO> oldConnections = resultList.stream()
                .collect(Collectors.toMap(ConnectionPropertiesDTO::getId, i -> i));

        newConnectionList.forEach(connection -> {
            ConnectionPropertiesDTO connectionToUpdate = oldConnections.get(connection.getId());

            if (connectionToUpdate != null) {
                connectionToUpdate.getNameProperty().setValue(connection.getName());
                connectionToUpdate.getUrlProperty().setValue(connection.getUrl());
                connectionToUpdate.getPortProperty().setValue(connection.getPort());
                connectionToUpdate.getClientIdProperty().setValue(connection.getClientId());
                connectionToUpdate.getUsernameProperty().setValue(connection.getUsername());
                connectionToUpdate.getPasswordProperty().setValue(connection.getPassword());
                connectionToUpdate.getCleanSessionProperty().setValue(connection.isCleanSession());
                connectionToUpdate.getMqttVersionProperty().setValue(connection.getMqttVersion());
                connectionToUpdate.getSslProperty().setValue(connection.getSsl());
                connectionToUpdate.getSslKeystoreProperty().setValue(connection.getSslKeystore());
                connectionToUpdate.getSslKeystorePasswordProperty().setValue(connection.getSslKeystorePassword());
                connectionToUpdate.getProxyProperty().setValue(connection.getProxy());
                connectionToUpdate.getSshHostProperty().setValue(connection.getSshHost());
                connectionToUpdate.getSshPortProperty().setValue(connection.getSshPort());
                connectionToUpdate.getLocalPortProperty().setValue(connection.getLocalPort());
                connectionToUpdate.getAuthProperty().setValue(connection.getAuth());
                connectionToUpdate.getAuthUsernameProperty().setValue(connection.getAuthUsername());
                connectionToUpdate.getAuthPasswordProperty().setValue(connection.getAuthPassword());
                connectionToUpdate.getAuthKeyfileProperty().setValue(connection.getAuthKeyfile());
                connectionToUpdate.getLwtProperty().setValue(connection.getLwt());
                connectionToUpdate.getLwtTopicProperty().setValue(connection.getLwtTopic());
                connectionToUpdate.getLwtQoSProperty().setValue(connection.getLwtQos());
                connectionToUpdate.getLwtRetainedProperty().setValue(connection.isLwtRetained());
                connectionToUpdate.getLwtPayloadProperty().setValue(connection.getLwtPayload());
            } else {
                resultList.add(connection);
            }
        });

        List<ConnectionPropertiesDTO> allConnectionsToDelete = new ArrayList<>();
        Map<String, ConnectionPropertiesDTO> newConnections = newConnectionList.stream()
                .collect(Collectors.toMap(ConnectionPropertiesDTO::getId, i -> i));

        resultList.forEach(connection -> {
            ConnectionPropertiesDTO connectionToDelete = newConnections.get(connection.getId());

            if (connectionToDelete == null) {
                allConnectionsToDelete.add(connection);
            }
        });

        if (!allConnectionsToDelete.isEmpty()) {
            resultList.removeAll(allConnectionsToDelete);
        }

        connectionListView.getItems().clear();
        connectionListView.getItems().addAll(resultList.stream()
                .sorted(Comparator.comparing(o -> sorted.get(o.getId())))
                .toList());

        boolean isEmpty = connectionListView.getItems().isEmpty();
        connectionListView.setVisible(!isEmpty);
        connectionListView.setManaged(!isEmpty);
        noConnectionsLabel.setVisible(isEmpty);
        noConnectionsLabel.setManaged(isEmpty);
        buttonBar.setVisible(!isEmpty);
        buttonBar.setManaged(!isEmpty);
        noConnectionsButtonBar.setVisible(isEmpty);
        noConnectionsButtonBar.setManaged(isEmpty);

        connectionListView.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> updateConnectButton());
        updateConnectButton();
        connectionListView.getSelectionModel().selectFirst();

        LOGGER.debug("Updated connections");
    }

    @FXML
    public void addConnection(ActionEvent actionEvent) {
        openSettings();
    }

    private void updateConnectButton() {
        ConnectionPropertiesDTO selectedConfig = connectionListView.getSelectionModel().getSelectedItem();
        helloViewConnectButton.setDisable(selectedConfig == null);
    }

    @FXML
    public void onClickConnect(ActionEvent actionEvent) {
        LOGGER.debug("Clicked on connect button");
        connect();
    }

    private void connect() {
        ConnectionPropertiesDTO config = connectionListView.getSelectionModel().getSelectedItem();
        connectionsOnboardingDelegate.onConnect(config);
    }

    @FXML
    public void openSettings() {
        ConnectionSettingsViewController.showAsDialog(connectionsSettingsViewDelegate, connectionListView.getSelectionModel().getSelectedItem());
        LOGGER.debug("Open connection settings");
    }

    @Override
    public void onConfigDirectoryEmpty() {
        // nothing to do
    }

    @Override
    public void onConfigDirectoryNotAccessible() {
        // nothing to do
    }

    @Override
    public void onAppDataNull() {
        // nothing to do
    }

    @Override
    public void onUserHomeNull() {
        // nothing to do
    }

    @Override
    public void onFileAlreadyExists() {
        // nothing to do
    }

    @Override
    public void onInvalidPath() {
        // nothing to do
    }

    @Override
    public void onInvalidJsonFormat() {
        // nothing to do
    }

    @Override
    public void onSavingFailed() {
        // nothing to do
    }

    @Override
    public void onSettingsUpdated(boolean showRestartRequiredDialog) {
        // nothing to do
    }

    @Override
    public void onConnectionsUpdated() {
        updateConnections();
    }

    @Override
    public void onConfigPrepareFailed() {
        // nothing to do
    }

    public void cleanUp() {
        ConnectionPropertiesDTO config = connectionListView.getSelectionModel().getSelectedItem();
        connectionsOnboardingDelegate.cleanUpProvider(config);
        ConfigDispatcher.getInstance().removeObserver(this);
    }
}
