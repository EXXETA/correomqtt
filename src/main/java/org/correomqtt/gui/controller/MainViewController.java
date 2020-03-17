package org.correomqtt.gui.controller;

import org.correomqtt.business.dispatcher.ConfigDispatcher;
import org.correomqtt.business.dispatcher.ConfigObserver;
import org.correomqtt.business.services.ConfigService;
import org.correomqtt.business.services.PersistPublishHistoryService;
import org.correomqtt.business.services.PersistPublishMessageHistoryService;
import org.correomqtt.business.services.PersistSubscriptionHistoryService;
import org.correomqtt.business.utils.ConnectionHolder;
import org.correomqtt.gui.helper.AlertHelper;
import org.correomqtt.gui.model.ConnectionPropertiesDTO;
import org.correomqtt.gui.model.ConnectionState;
import org.correomqtt.gui.transformer.ConnectionTransformer;
import org.correomqtt.gui.utils.CheckNewVersionUtils;
import org.correomqtt.gui.utils.HostServicesHolder;
import javafx.collections.FXCollections;
import javafx.collections.ObservableList;
import javafx.fxml.FXML;
import javafx.scene.control.Menu;
import javafx.scene.control.MenuBar;
import javafx.scene.control.MenuItem;
import javafx.scene.control.*;
import javafx.scene.input.KeyEvent;
import javafx.scene.layout.AnchorPane;
import org.json.simple.parser.ParseException;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;
import java.util.ResourceBundle;
import java.util.UUID;

public class MainViewController implements ConnectionOnboardingDelegate, ConnectionViewDelegate, ConfigObserver, ConnectionSettingsViewDelegate {

    private static final Logger LOGGER = LoggerFactory.getLogger(MainViewController.class);

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

    public MainViewController() {
        ConfigDispatcher.getInstance().addObserver(this);
    }

    @FXML
    public void initialize() {
        tabPaneAnchorPane.getStyleClass().add(ConfigService.getInstance().getThemeSettings().getActiveTheme().getIconMode());

        tabPane.setTabClosingPolicy(TabPane.TabClosingPolicy.ALL_TABS);
        setupAddTab();
        createLogTab();

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
        LoaderResult<ConnectionOnbordingViewController> loadResult = ConnectionOnbordingViewController.load(this, this);
        addTab.setContent(loadResult.getMainPane());
        resources = loadResult.getResourceBundle();

        selectionModel = tabPane.getSelectionModel();
        selectionModel.select(addTab);
    }

    private void createLogTab() {
        LoaderResult<LogTabController> result = LogTabController.load();
        LogTabController logViewController = result.getController();
        logTab.setClosable(false);
        logAnchorPane.getChildren().add(logViewController.logViewAnchor);
        logViewController.logViewAnchor.prefWidthProperty().bind(logAnchorPane.widthProperty());
        logViewController.logViewAnchor.prefHeightProperty().bind(logAnchorPane.heightProperty());
    }

    private void setMenuEventHandler() {
        closeItem.setOnAction(event -> System.exit(0));
        connectionsItem.setOnAction(event -> ConnectionSettingsViewController.showAsDialog(this));
        settingsItem.setOnAction(event -> SettingsViewController.showAsDialog());
        aboutItem.setOnAction(event -> AboutViewController.showAsDialog());
        updateItem.setOnAction(event -> {
            try {
                CheckNewVersionUtils.checkNewVersion(true);
            } catch (IOException e) {
                e.printStackTrace();
            } catch (ParseException e) {
                e.printStackTrace();
            }
        });
        websiteItem.setOnAction(event -> {
            //TODO: Replace with github page
            HostServicesHolder.getInstance().getHostServices().showDocument(
                    new Hyperlink("https://github.com/EXXETA/correomqtt").getText());
        });
        pluginSettingsItem.setOnAction(event -> openPluginSettings());
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

            PersistPublishHistoryService.activate(config.getId());
            PersistPublishMessageHistoryService.activate(config.getId());
            PersistSubscriptionHistoryService.activate(config.getId());

            Tab tab = new Tab();
            tab.setId(tabId);
            tab.setClosable(true);
            tab.setText(config.getName());
            tab.setOnSelectionChanged(event -> {
                if (tab.isSelected()) {
                    tab.getStyleClass().removeAll("dirty");
                }
            });
            tab.getStyleClass().add("connection");

            config.getNameProperty().addListener(((observableValue, s, t1) -> {
                tab.setText(t1);
            }));

            LoaderResult<ConnectionViewController> result = ConnectionViewController.load(config.getId(), this);
            result.getController().setTabId(tabId);
            tab.setContent(result.getMainPane());
            tab.setOnCloseRequest(event -> result.getController().disconnect());

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

    private void calcTabWidth() {
        ObservableList<Tab> tabs = FXCollections.observableArrayList(tabPane.getTabs());
        tabPane.setTabMaxWidth((tabPane.widthProperty().doubleValue() - 80) / Math.max(1, tabs.size() - 2) - 25);
    }

    @Override
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
                        t.getStyleClass().removeAll("dirty");
                        t.getStyleClass().add("dirty");
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
    public void onConfigDirectoryEmpty() {

    }

    @Override
    public void onConfigDirectoryNotAccessible() {

    }

    @Override
    public void onAppDataNull() {

    }

    @Override
    public void onUserHomeNull() {

    }

    @Override
    public void onFileAlreadyExists() {

    }

    @Override
    public void onInvalidPath() {

    }

    @Override
    public void onInvalidJsonFormat() {

    }

    @Override
    public void onSavingFailed() {

    }

    @Override
    public void onSettingsUpdated() {

    }

    @Override
    public void onConnectionsUpdated() {
    }

    @Override
    public void onConfigPrepareFailed() {

    }

    @Override
    public void closeTab(String connectionName) {
        tabPane.getTabs().stream()
                .filter(t -> connectionName.equals(t.getText()))
                .findFirst()
                .ifPresent(t -> {
                    tabPane.getTabs().remove(t);
                });
        LOGGER.info("Closing tab for connection: " + connectionName);
    }
}
