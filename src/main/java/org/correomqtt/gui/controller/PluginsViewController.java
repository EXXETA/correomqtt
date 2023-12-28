package org.correomqtt.gui.controller;

import javafx.fxml.FXML;
import javafx.scene.control.CheckBox;
import javafx.scene.control.Label;
import javafx.scene.control.ListView;
import javafx.scene.control.Tab;
import javafx.scene.control.TableColumn;
import javafx.scene.control.TableView;
import org.correomqtt.business.provider.PluginConfigProvider;
import org.correomqtt.gui.model.PluginInfoPropertiesDTO;
import org.correomqtt.gui.model.WindowProperty;
import org.correomqtt.gui.model.WindowType;
import org.correomqtt.gui.utils.HostServicesHolder;
import org.correomqtt.gui.utils.WindowHelper;
import org.pf4j.PluginWrapper;

import java.io.File;
import java.util.HashMap;
import java.util.Map;
import java.util.ResourceBundle;

public class PluginsViewController extends BaseControllerImpl {

    @FXML
    private Tab marketplaceTab;

    @FXML
    private TableView<PluginWrapper> pluginsTableView;

    @FXML
    private TableColumn<PluginWrapper, CheckBox> isEnabledColumn;

    @FXML
    private TableColumn<PluginWrapper, String> nameVersionColumn;

    @FXML
    private TableColumn<PluginWrapper, String> descriptionColumn;

    @FXML
    private TableColumn<PluginWrapper, String> providerColumn;

    @FXML
    private TableColumn<PluginWrapper, String> permissionColumn;

    @FXML
    private TableColumn<PluginWrapper, String> fileColumn;

    @FXML
    private Label statusText;

    @FXML
    private ListView<PluginInfoPropertiesDTO> marketplacePluginList;

    @FXML
    private Tab installedPluginsTab;

    public static void showAsDialog() {
        Map<Object, Object> properties = new HashMap<>();
        properties.put(WindowProperty.WINDOW_TYPE, WindowType.PLUGIN_SETTINGS);

        if (WindowHelper.focusWindowIfAlreadyThere(properties)) {
            return;
        }

        LoaderResult<PluginsViewController> result = load(PluginsViewController.class, "pluginsView.fxml");
        ResourceBundle resources = result.getResourceBundle();
        showAsDialog(result,
                resources.getString("pluginsViewControllerTitle"),
                properties,
                true,
                false,
                null,
                null,
                500,
                400);
    }

    @FXML
    public void initialize() {

        setupInstalledPluginTab();
        setupMarketplaceTab();
    }

    private void setupInstalledPluginTab(){
        LoaderResult<InstalledPluginsViewController> result = InstalledPluginsViewController.load();
        installedPluginsTab.setContent(result.getMainRegion());
    }

    private void setupMarketplaceTab(){
        LoaderResult<MarketplaceViewController> result = MarketplaceViewController.load();
        marketplaceTab.setContent(result.getMainRegion());
    }

    @FXML
    public void onOpenPluginFolder() {
        HostServicesHolder.getInstance().getHostServices().showDocument(new File(PluginConfigProvider.getInstance().getPluginPath()).toURI().toString());
    }
}
