package org.correomqtt.gui.views;

import javafx.collections.FXCollections;
import javafx.collections.ObservableList;
import javafx.fxml.FXML;
import javafx.scene.control.Hyperlink;
import javafx.scene.control.Menu;
import javafx.scene.control.MenuBar;
import javafx.scene.control.MenuItem;
import javafx.scene.control.SelectionModel;
import javafx.scene.control.Tab;
import javafx.scene.control.TabPane;
import javafx.scene.input.KeyEvent;
import javafx.scene.layout.AnchorPane;
import lombok.Getter;
import org.correomqtt.core.applifecycle.ShutdownRequestEvent;
import org.correomqtt.core.connection.ConnectionStateChangedEvent;
import org.correomqtt.core.eventbus.EventBus;
import org.correomqtt.core.eventbus.Subscribe;
import org.correomqtt.core.exception.CorreoMqttUnableToCheckVersionException;
import org.correomqtt.core.fileprovider.PersistPublishHistoryProvider;
import org.correomqtt.core.fileprovider.PersistPublishMessageHistoryProvider;
import org.correomqtt.core.fileprovider.PersistSubscriptionHistoryProvider;
import org.correomqtt.business.settings.SettingsProvider;
import org.correomqtt.core.utils.ConnectionHolder;
import org.correomqtt.core.utils.VendorConstants;
import org.correomqtt.gui.controls.ThemedFontIcon;
import org.correomqtt.gui.model.ConnectionPropertiesDTO;
import org.correomqtt.gui.model.GuiConnectionState;
import org.correomqtt.gui.transformer.ConnectionTransformer;
import org.correomqtt.gui.utils.CheckNewVersionUtils;
import org.correomqtt.gui.utils.HostServicesHolder;
import org.correomqtt.gui.views.about.AboutViewController;
import org.correomqtt.gui.views.connections.ConnectionViewController;
import org.correomqtt.gui.views.connections.ConnectionViewControllerFactory;
import org.correomqtt.gui.views.connections.ConnectionViewDelegate;
import org.correomqtt.gui.views.connectionsettings.ConnectionSettingsViewController;
import org.correomqtt.gui.views.connectionsettings.ConnectionSettingsViewDelegate;
import org.correomqtt.gui.views.importexport.ConnectionExportViewController;
import org.correomqtt.gui.views.importexport.ConnectionImportViewController;
import org.correomqtt.gui.views.log.LogTabController;
import org.correomqtt.gui.views.onboarding.ConnectionOnboardingDelegate;
import org.correomqtt.gui.views.onboarding.ConnectionOnbordingViewController;
import org.correomqtt.gui.views.plugins.PluginsViewController;
import org.correomqtt.gui.views.scripting.ScriptingViewController;
import org.correomqtt.gui.views.settings.SettingsViewController;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import javax.inject.Inject;
import java.io.IOException;
import java.util.HashMap;
import java.util.Map;


public class MainViewController implements ConnectionOnboardingDelegate, ConnectionViewDelegate, ConnectionSettingsViewDelegate {

    private static final Logger LOGGER = LoggerFactory.getLogger(MainViewController.class);
    public static final String DIRTY_CLASS = "dirty";
    private final ConnectionHolder connectionHolder;
    private final ConnectionViewControllerFactory connectionViewControllerFactory;

    @FXML
    @Getter
    private TabPane tabPane;
    @FXML
    private Tab logTab;
    @FXML
    private AnchorPane logAnchorPane;
    @FXML
    private Tab addTab;
    @FXML
    private AnchorPane tabPaneAnchorPane;
    @FXML
    private MenuItem exportConnectionsItem;
    @FXML
    private MenuItem importConnectionsItem;
    @FXML
    private MenuBar menuBar;
    @FXML
    private Menu fileMenu;
    @FXML
    private MenuItem closeItem;
    @FXML
    private MenuItem connectionsItem;
    @FXML
    private MenuItem settingsItem;
    @FXML
    private MenuItem aboutItem;
    @FXML
    private MenuItem updateItem;
    @FXML
    private MenuItem websiteItem;
    @FXML
    private MenuItem pluginSettingsItem;
    @FXML
    private MenuItem scriptingItem;

    private SelectionModel<Tab> selectionModel;

    @Getter
    private Map<String, ConnectionViewController> connectionViewControllers;

    private ConnectionOnbordingViewController connectionOnboardingViewController;

    private LogTabController logViewController;

    private String closedTabId;

    @Inject
    public MainViewController(ConnectionHolder connectionHolder,
                              ConnectionViewControllerFactory connectionViewControllerFactory) {
        this.connectionHolder = connectionHolder;
        this.connectionViewControllerFactory = connectionViewControllerFactory;
        EventBus.register(this);
    }


    @FXML
    private void initialize() {
        tabPaneAnchorPane.getStyleClass().add(SettingsProvider.getInstance().getIconModeCssClass());

        tabPane.setTabClosingPolicy(TabPane.TabClosingPolicy.ALL_TABS);
        setupAddTab();
        createLogTab();

        connectionViewControllers = new HashMap<>();

        final String os = System.getProperty("os.name");
        if (os != null && os.startsWith("Mac")) {
            menuBar.getMenus().remove(fileMenu);
            menuBar.useSystemMenuBarProperty().set(true);
            AnchorPane.setTopAnchor(tabPane, 0.0);
            System.setProperty("apple.laf.useScreenMenuBar", "true");
        }

        setMenuEventHandler();

        tabPane.addEventHandler(KeyEvent.KEY_PRESSED, event -> {
            if (event.getCode().isDigitKey() && event.isShortcutDown()) {
                selectionModel.select(Integer.parseInt(event.getText()) - 1);
            }
        });

        tabPane.widthProperty().addListener((a, b, c) -> calcTabWidth());
    }

    private void setupAddTab() {
        addTab.setClosable(false);
        addTab.setGraphic(new ThemedFontIcon("mdi-home"));
        LoaderResult<ConnectionOnbordingViewController> loadResult = ConnectionOnbordingViewController.load(this, this);
        addTab.setContent(loadResult.getMainRegion());
        connectionOnboardingViewController = loadResult.getController();

        selectionModel = tabPane.getSelectionModel();
        selectionModel.select(addTab);
    }

    private void createLogTab() {
        LoaderResult<LogTabController> result = LogTabController.load();
        logViewController = result.getController();
        logTab.setClosable(false);
        logTab.setGraphic(new ThemedFontIcon("mdi-chart-box"));
        logAnchorPane.getChildren().add(logViewController.getLogViewAnchor());
        logViewController.getLogViewAnchor().prefWidthProperty().bind(logAnchorPane.widthProperty());
        logViewController.getLogViewAnchor().prefHeightProperty().bind(logAnchorPane.heightProperty());
    }

    private void setMenuEventHandler() {
        closeItem.setOnAction(event -> EventBus.fireAsync(new ShutdownRequestEvent()));
        connectionsItem.setOnAction(event -> ConnectionSettingsViewController.showAsDialog(this, null));
        settingsItem.setOnAction(event -> SettingsViewController.showAsDialog());
        aboutItem.setOnAction(event -> AboutViewController.showAsDialog());
        updateItem.setOnAction(event -> {
            try {
                CheckNewVersionUtils.checkNewVersion(true);
            } catch (IOException | CorreoMqttUnableToCheckVersionException e) {
                LOGGER.warn("Exception checking version: {}", e.getMessage());
            }
        });
        scriptingItem.setOnAction(event -> ScriptingViewController.showAsDialog());
        websiteItem.setOnAction(event -> HostServicesHolder.getInstance().getHostServices().showDocument(new Hyperlink(VendorConstants.WEBSITE()).getText()));
        pluginSettingsItem.setOnAction(event -> openPluginSettings());
        exportConnectionsItem.setOnAction(event -> ConnectionExportViewController.showAsDialog());
        importConnectionsItem.setOnAction(event -> ConnectionImportViewController.showAsDialog());
    }

    private void openPluginSettings() {
        PluginsViewController.showAsDialog();
    }

    public String getUUIDofSelectedTab() {
        Tab selectedTab = selectionModel.getSelectedItem();
        return selectedTab.getId();
    }

    @Override
    public void setTabName(String tabId, String name) {
        tabPane.getTabs().stream().filter(t -> t.getId().equals(tabId)).findFirst().ifPresent(t -> t.setText(name));
        calcTabWidth();
    }

    @SuppressWarnings("unused")
    public void onConnectionStateChanged(@Subscribe ConnectionStateChangedEvent event) {
        if (connectionViewControllers.values().stream().noneMatch(ctrl -> ctrl.getConnectionId().equals(event.getConnectionId()))) {
            getConnectionViewControllerLoaderResult(ConnectionTransformer.dtoToProps(connectionHolder.getConfig(event.getConnectionId())));
        }
    }

    @Override
    public void onConnect(ConnectionPropertiesDTO config) {
        Tab connectTab = tabPane.getTabs().stream()
                .filter(tab -> tab.getId().equals(config.getId()))
                .findFirst()
                .orElse(null);
        if (connectTab == null) {
            LoaderResult<ConnectionViewController> result = getConnectionViewControllerLoaderResult(config);
            result.getController().connect(config);

        } else {
            tabPane.getSelectionModel().select(connectTab);
        }

    }

    private LoaderResult<ConnectionViewController> getConnectionViewControllerLoaderResult(ConnectionPropertiesDTO config) {
        String tabId = config.getId();

        PersistPublishHistoryProvider.activate(config.getId());
        PersistPublishMessageHistoryProvider.activate(config.getId());
        PersistSubscriptionHistoryProvider.activate(config.getId());

        Tab tab = new Tab();
        tab.setId(tabId);
        tab.setGraphic(new ThemedFontIcon("correo-wifi-solid"));
        tab.setClosable(true);
        tab.setText(config.getName());
        tab.setOnSelectionChanged(event -> {
            if (tab.isSelected()) {
                tab.getStyleClass().removeAll(DIRTY_CLASS);
            }
        });

        config.getNameProperty().addListener(((observableValue, s, t1) -> tab.setText(t1)));

        LoaderResult<ConnectionViewController> result = connectionViewControllerFactory.create(config.getId(), this).load();
        result.getController().setTabId(tabId);
        tab.setContent(result.getMainRegion());
        tab.setOnCloseRequest(event -> this.onTabClose(result, tabId));

        connectionViewControllers.put(tabId, result.getController());

        tabPane.getTabs().add(tabPane.getTabs().size() - 1, tab);
        selectionModel = tabPane.getSelectionModel();
        selectionModel.select(tab);

        LOGGER.debug("New tab created");
        calcTabWidth();
        return result;
    }

    private void onTabClose(LoaderResult<ConnectionViewController> result, String tabId) {
        closedTabId = tabId;
        result.getController().close();
        connectionViewControllers.remove(closedTabId);
    }

    @FXML
    private void resetUISettings() {
        if (connectionViewControllers.get(tabPane.getSelectionModel().getSelectedItem().getId()) != null) {
            connectionViewControllers.get(tabPane.getSelectionModel().getSelectedItem().getId()).resetConnectionUISettings();
        }
    }

    @FXML
    private void onClickP() {
        if (connectionViewControllers.get(tabPane.getSelectionModel().getSelectedItem().getId()) != null) {
            connectionViewControllers.get(tabPane.getSelectionModel().getSelectedItem().getId()).setLayout(true, false);
        }
    }

    @FXML
    private void onClickPS() {
        if (connectionViewControllers.get(tabPane.getSelectionModel().getSelectedItem().getId()) != null) {
            connectionViewControllers.get(tabPane.getSelectionModel().getSelectedItem().getId()).setLayout(true, true);
        }
    }

    @FXML
    private void onClickS() {
        if (connectionViewControllers.get(tabPane.getSelectionModel().getSelectedItem().getId()) != null) {
            connectionViewControllers.get(tabPane.getSelectionModel().getSelectedItem().getId()).setLayout(false, true);
        }
    }

    private void calcTabWidth() {
        ObservableList<Tab> tabs = FXCollections.observableArrayList(tabPane.getTabs());
        tabPane.setTabMaxWidth((tabPane.widthProperty().doubleValue() - 80) / Math.max(1, tabs.size() - 2) - 25);
    }

    @Override
    public void onCleanup() {
        ConnectionViewController connectionViewController = connectionViewControllers.get(this.closedTabId);
        connectionViewController.cleanUp();
        connectionOnboardingViewController.cleanUp();
        logViewController.cleanUp();
        connectionViewControllers.remove(this.closedTabId);
        EventBus.unregister(this);
    }

    @Override
    public void cleanUpProvider(ConnectionPropertiesDTO config) {
        PersistPublishHistoryProvider.getInstance(config.getId()).cleanUp();
        PersistPublishMessageHistoryProvider.getInstance(config.getId()).cleanUp();
        PersistSubscriptionHistoryProvider.getInstance(config.getId()).cleanUp();
    }

    public void onDisconnect() {
        calcTabWidth();
    }

    @Override
    public void setTabDirty(String tabId) {
        tabPane.getTabs().stream().filter(t -> tabId.equals(t.getId())).findFirst().ifPresent(t -> {
            if (!t.isSelected()) {
                t.getStyleClass().removeAll(DIRTY_CLASS);
                t.getStyleClass().add(DIRTY_CLASS);
            }
        });
    }

    @Override
    public void setConnectionState(String tabId, GuiConnectionState state) {
        tabPane.getTabs()
                .stream()
                .filter(t -> t.getId().equals(tabId))
                .findFirst()
                .ifPresent(t -> ((ThemedFontIcon) t.getGraphic()).setIconColor(state.getIconColor()));
    }

    @Override
    public void closeTab(String connectionName) {
        tabPane.getTabs().stream().filter(t -> connectionName.equals(t.getText())).findFirst().ifPresent(t -> tabPane.getTabs().remove(t));
        LOGGER.info("Closing tab for connection: {}", connectionName);
    }
}
