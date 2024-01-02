package org.correomqtt.gui.controller;

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
import org.correomqtt.business.eventbus.EventBus;
import org.correomqtt.business.exception.CorreoMqttUnableToCheckVersionException;
import org.correomqtt.business.fileprovider.PersistPublishHistoryProvider;
import org.correomqtt.business.fileprovider.PersistPublishMessageHistoryProvider;
import org.correomqtt.business.fileprovider.PersistSubscriptionHistoryProvider;
import org.correomqtt.business.fileprovider.SettingsProvider;
import org.correomqtt.business.utils.ConnectionHolder;
import org.correomqtt.business.applifecycle.ShutdownRequestEvent;
import org.correomqtt.business.utils.VendorConstants;
import org.correomqtt.gui.helper.AlertHelper;
import org.correomqtt.gui.model.ConnectionPropertiesDTO;
import org.correomqtt.gui.model.ConnectionState;
import org.correomqtt.gui.transformer.ConnectionTransformer;
import org.correomqtt.gui.utils.CheckNewVersionUtils;
import org.correomqtt.gui.utils.HostServicesHolder;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;
import java.util.HashMap;
import java.util.Map;
import java.util.ResourceBundle;
import java.util.UUID;


public class MainViewController implements ConnectionOnboardingDelegate, ConnectionViewDelegate, ConnectionSettingsViewDelegate {

    private static final Logger LOGGER = LoggerFactory.getLogger(MainViewController.class);
    public static final String DIRTY_CLASS = "dirty";

    @FXML
    public TabPane tabPane;

    @FXML
    public Tab logTab;
    @FXML
    public AnchorPane logAnchorPane;
    @FXML
    public Tab addTab;
    @FXML
    public AnchorPane tabPaneAnchorPane;
    @FXML
    public MenuItem exportConnectionsItem;
    @FXML
    public MenuItem importConnectionsItem;
    @FXML
    private MenuBar menuBar;
    @FXML
    private Menu fileMenu;
    @FXML
    private MenuItem closeItem;
    @FXML
    private Menu toolsMenu;
    @FXML
    private MenuItem connectionsItem;
    @FXML
    private MenuItem settingsItem;
    @FXML
    private Menu helpMenu;
    @FXML
    private MenuItem aboutItem;
    @FXML
    private MenuItem updateItem;
    @FXML
    private MenuItem websiteItem;
    @FXML
    private Menu pluginMenu;
    @FXML
    private MenuItem pluginSettingsItem;

    private SelectionModel<Tab> selectionModel;
    private ResourceBundle resources;
    private Map<String, ConnectionViewController> conntectionViewControllers;

    private ConnectionOnbordingViewController connectionOnboardingViewController;

    private LogTabController logViewController;

    private String closedTabId;

    public MainViewController() {
    }

    @FXML
    public void initialize() {
        tabPaneAnchorPane.getStyleClass().add(SettingsProvider.getInstance().getIconModeCssClass());

        tabPane.setTabClosingPolicy(TabPane.TabClosingPolicy.ALL_TABS);
        setupAddTab();
        createLogTab();

        conntectionViewControllers = new HashMap<>();

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

    public Map<String, ConnectionViewController> getConntectionViewControllers() {
        return conntectionViewControllers;
    }

    private void setupAddTab() {
        addTab.setClosable(false);
        LoaderResult<ConnectionOnbordingViewController> loadResult = ConnectionOnbordingViewController.load(this,this);
        addTab.setContent(loadResult.getMainRegion());
        resources = loadResult.getResourceBundle();
        connectionOnboardingViewController = loadResult.getController();

        selectionModel = tabPane.getSelectionModel();
        selectionModel.select(addTab);
    }

    private void createLogTab() {
        LoaderResult<LogTabController> result = LogTabController.load();
        logViewController = result.getController();
        logTab.setClosable(false);
        logAnchorPane.getChildren().add(logViewController.logViewAnchor);
        logViewController.logViewAnchor.prefWidthProperty().bind(logAnchorPane.widthProperty());
        logViewController.logViewAnchor.prefHeightProperty().bind(logAnchorPane.heightProperty());
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
                LOGGER.warn("Exception checking version", e); //TODO UI?
            }
        });
        websiteItem.setOnAction(event -> HostServicesHolder.getInstance().getHostServices().showDocument(
                new Hyperlink(VendorConstants.WEBSITE()).getText()));
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
        tabPane.getTabs().stream()
                .filter(t -> t.getId().equals(tabId))
                .findFirst()
                .ifPresent(t -> t.setText(name));
        calcTabWidth();
    }

    @Override
    public void onConnect(ConnectionPropertiesDTO config) {
        if (ConnectionHolder.getInstance().isConnectionUnused(ConnectionTransformer.propsToDto(config))) {
            String tabId = UUID.randomUUID().toString();

            PersistPublishHistoryProvider.activate(config.getId());
            PersistPublishMessageHistoryProvider.activate(config.getId());
            PersistSubscriptionHistoryProvider.activate(config.getId());

            Tab tab = new Tab();
            tab.setId(tabId);
            tab.setClosable(true);
            tab.setText(config.getName());
            tab.setOnSelectionChanged(event -> {
                if (tab.isSelected()) {
                    tab.getStyleClass().removeAll(DIRTY_CLASS);
                }
            });
            tab.getStyleClass().add("connection");

            config.getNameProperty().addListener(((observableValue, s, t1) -> tab.setText(t1)));

            LoaderResult<ConnectionViewController> result = ConnectionViewController.load(config.getId(), this);
            result.getController().setTabId(tabId);
            tab.setContent(result.getMainRegion());
            tab.setOnCloseRequest(event -> this.onTabClose(result, tabId));

            conntectionViewControllers.put(tabId, result.getController());
            System.out.println("Main connect: " + conntectionViewControllers.toString());

            tabPane.getTabs().add(tabPane.getTabs().size() - 1, tab);
            selectionModel = tabPane.getSelectionModel();
            selectionModel.select(tab);

            LOGGER.debug("New tab created");

            result.getController().connect(config);

            calcTabWidth();
        } else {
            AlertHelper.warn(resources.getString("mainViewControllerAlreadyUsedTitle"),
                    resources.getString("mainViewControllerAlreadyUsedContent"));
        }

    }

    private void onTabClose(LoaderResult<ConnectionViewController> result, String tabId) {
        closedTabId = tabId;
        result.getController().disconnect(true);
    }

    @FXML
    public void resetUISettings() {
        if (conntectionViewControllers.get(tabPane.getSelectionModel().getSelectedItem().getId()) != null) {
            conntectionViewControllers.get(tabPane.getSelectionModel().getSelectedItem().getId()).resetConnectionUISettings();
        }
    }

    @FXML
    public void onClickP() {
        if (conntectionViewControllers.get(tabPane.getSelectionModel().getSelectedItem().getId()) != null) {
            conntectionViewControllers.get(tabPane.getSelectionModel().getSelectedItem().getId()).setLayout(true, false);
        }
    }

    @FXML
    public void onClickPS() {
        if (conntectionViewControllers.get(tabPane.getSelectionModel().getSelectedItem().getId()) != null) {
            conntectionViewControllers.get(tabPane.getSelectionModel().getSelectedItem().getId()).setLayout(true, true);
        }
    }

    @FXML
    public void onClickS() {
        if (conntectionViewControllers.get(tabPane.getSelectionModel().getSelectedItem().getId()) != null) {
            conntectionViewControllers.get(tabPane.getSelectionModel().getSelectedItem().getId()).setLayout(false, true);
        }
    }

    private void calcTabWidth() {
        ObservableList<Tab> tabs = FXCollections.observableArrayList(tabPane.getTabs());
        tabPane.setTabMaxWidth((tabPane.widthProperty().doubleValue() - 80) / Math.max(1, tabs.size() - 2) - 25);
    }

    @Override
    public void onCleanup() {
        ConnectionViewController connectionViewController = conntectionViewControllers.get(this.closedTabId);
        connectionViewController.cleanUp();
        connectionOnboardingViewController.cleanUp();
        logViewController.cleanUp();
        conntectionViewControllers.remove(this.closedTabId);
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
        tabPane.getTabs().stream()
                .filter(t -> tabId.equals(t.getId()))
                .findFirst()
                .ifPresent(t -> {
                    if (!t.isSelected()) {
                        t.getStyleClass().removeAll(DIRTY_CLASS);
                        t.getStyleClass().add(DIRTY_CLASS);
                    }
                });
    }

    @Override
    public void setConnectionState(String tabId, ConnectionState state) {
        tabPane.getTabs().stream()
                .filter(t -> t.getId().equals(tabId))
                .findFirst()
                .ifPresent(t -> {
                    t.getStyleClass().removeAll("connected", "connecting", "disconnecting", "graceful", "ungraceful");
                    t.getStyleClass().add(state.getCssClass());
                });
    }

    @Override
    public void closeTab(String connectionName) {
        tabPane.getTabs().stream()
                .filter(t -> connectionName.equals(t.getText()))
                .findFirst()
                .ifPresent(t -> tabPane.getTabs().remove(t));
        LOGGER.info("Closing tab for connection: {}", connectionName);
    }
}
