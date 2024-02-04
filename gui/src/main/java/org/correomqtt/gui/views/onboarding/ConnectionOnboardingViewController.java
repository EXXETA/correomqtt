package org.correomqtt.gui.views.onboarding;

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
import org.correomqtt.core.CoreManager;
import org.correomqtt.core.connection.ConnectionStateChangedEvent;
import org.correomqtt.core.eventbus.EventBus;
import org.correomqtt.core.eventbus.Subscribe;
import org.correomqtt.core.fileprovider.ConnectionsUpdatedEvent;
import org.correomqtt.di.Assisted;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import org.correomqtt.gui.model.ConnectionPropertiesDTO;
import org.correomqtt.gui.theme.ThemeManager;
import org.correomqtt.gui.transformer.ConnectionTransformer;
import org.correomqtt.gui.views.LoaderResult;
import org.correomqtt.gui.views.base.BaseControllerImpl;
import org.correomqtt.gui.views.cell.ConnectionCell;
import org.correomqtt.gui.views.cell.ConnectionCellFactory;
import org.correomqtt.gui.views.connectionsettings.ConnectionSettingsViewControllerFactory;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.Map;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.stream.Collectors;

@DefaultBean
public class ConnectionOnboardingViewController extends BaseControllerImpl {

    private static final Logger LOGGER = LoggerFactory.getLogger(ConnectionOnboardingViewController.class);
    private final ConnectionCellFactory connectionCellFactory;
    private final ConnectionSettingsViewControllerFactory connectionSettingsViewControllerFactory;
    private final EventBus eventBus;
    private final ConnectionOnboardingDelegate connectionsOnboardingDelegate;
    @FXML
    private AnchorPane helloViewAnchor;
    @FXML
    private ComboBox<String> helloViewComboBox;
    @FXML
    private Button helloViewConnectButton;
    @FXML
    private Button editConnectionsButton;
    @FXML
    private HBox mainHBox;
    @FXML
    private Label noConnectionsLabel;
    @FXML
    private VBox helloViewVBox;
    @FXML
    private ListView<ConnectionPropertiesDTO> connectionListView;
    @FXML
    private HBox buttonBar;
    @FXML
    private HBox noConnectionsButtonBar;



    @Inject
    public ConnectionOnboardingViewController(CoreManager coreManager,
                                              ThemeManager themeManager,
                                              ConnectionCellFactory connectionCellFactory,
                                              ConnectionSettingsViewControllerFactory connectionSettingsViewControllerFactory,
                                              EventBus eventBus,
                                              @Assisted ConnectionOnboardingDelegate connectionsOnboardingDelegate) {
        super(coreManager, themeManager);
        this.connectionCellFactory = connectionCellFactory;
        this.connectionSettingsViewControllerFactory = connectionSettingsViewControllerFactory;
        this.eventBus = eventBus;
        this.connectionsOnboardingDelegate = connectionsOnboardingDelegate;
        eventBus.register(this);
    }

    public LoaderResult<ConnectionOnboardingViewController> load() {
        return load(ConnectionOnboardingViewController.class, "connectionOnboardingView.fxml", () -> this);
    }

    @FXML
    private void initialize() {

        connectionListView.setCellFactory(this::createCell);

        connectionListView.setItems(FXCollections.observableArrayList(
                ConnectionTransformer.dtoListToPropList(coreManager.getConnectionManager().getSortedConnections())
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
        ConnectionCell cell = connectionCellFactory.create(connectionListView);
        cell.setOnMouseClicked(event -> onCellClicked(event, cell.getItem()));
        return cell;
    }

    private void updateConnections() {
        List<ConnectionPropertiesDTO> resultList = new ArrayList<>(connectionListView.getItems());

        List<ConnectionPropertiesDTO> newConnectionList = ConnectionTransformer.dtoListToPropList(coreManager.getConnectionManager().getSortedConnections());

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

    private void connect() {
        ConnectionPropertiesDTO config = connectionListView.getSelectionModel().getSelectedItem();
        connectionsOnboardingDelegate.onConnect(config);
    }

    private void onCellClicked(MouseEvent event, ConnectionPropertiesDTO configDTO) {
        if (configDTO != null && event.getButton().equals(MouseButton.PRIMARY) && event.getClickCount() == 2) {
            connect();
            if (LOGGER.isDebugEnabled()) {
                LOGGER.debug("Clicked on connect: {}", configDTO.getId());
            }
        }
    }

    private void updateConnectButton() {
        ConnectionPropertiesDTO selectedConfig = connectionListView.getSelectionModel().getSelectedItem();
        helloViewConnectButton.setDisable(selectedConfig == null);
    }

    @FXML
    private void addConnection(ActionEvent actionEvent) {
        openSettings();
    }

    @FXML
    private void openSettings() {
        connectionSettingsViewControllerFactory.create(connectionListView.getSelectionModel().getSelectedItem()).showAsDialog();
        LOGGER.debug("Open connection settings");
    }

    @FXML
    private void onClickConnect(ActionEvent actionEvent) {
        LOGGER.debug("Clicked on connect button");
        connect();
    }

    @SuppressWarnings("unused")
    @Subscribe(ConnectionsUpdatedEvent.class)
    public void onConnectionsUpdated() {
        updateConnections();
    }

    @SuppressWarnings("unused")
    @Subscribe(ConnectionStateChangedEvent.class)
    public void onConnectionStateChanged() {
        connectionListView.refresh();
    }

    public void cleanUp() {
        ConnectionPropertiesDTO config = connectionListView.getSelectionModel().getSelectedItem();
        connectionsOnboardingDelegate.cleanUpProvider(config);
        eventBus.unregister(this);
    }
}
