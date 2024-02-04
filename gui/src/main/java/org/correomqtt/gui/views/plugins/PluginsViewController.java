package org.correomqtt.gui.views.plugins;

import javafx.application.HostServices;
import javafx.fxml.FXML;
import javafx.scene.control.CheckBox;
import javafx.scene.control.Label;
import javafx.scene.control.ListView;
import javafx.scene.control.Tab;
import javafx.scene.control.TableColumn;
import javafx.scene.control.TableView;
import org.correomqtt.HostServicesWrapper;
import org.correomqtt.core.CoreManager;
import org.correomqtt.core.fileprovider.PluginConfigProvider;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import org.correomqtt.di.Lazy;
import org.correomqtt.gui.model.PluginInfoPropertiesDTO;
import org.correomqtt.gui.model.WindowProperty;
import org.correomqtt.gui.model.WindowType;
import org.correomqtt.gui.theme.ThemeManager;
import org.correomqtt.gui.utils.WindowHelper;
import org.correomqtt.gui.views.LoaderResult;
import org.correomqtt.gui.views.base.BaseControllerImpl;
import org.pf4j.PluginWrapper;

import java.io.File;
import java.util.HashMap;
import java.util.Map;
import java.util.ResourceBundle;

@DefaultBean
public class PluginsViewController extends BaseControllerImpl {

    private final PluginConfigProvider pluginConfigProvider;
    private final HostServices hostServices;
    private final Lazy<InstalledPluginsViewController> installedPluginsViewControllerProvider;
    private final Lazy<MarketplaceViewController> marketplaceViewControllerProvider;
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

    @Inject
    PluginsViewController(CoreManager coreManager,
                          ThemeManager themeManager,
                          PluginConfigProvider pluginConfigProvider,
                          HostServicesWrapper hostServicesWrapper,
                          Lazy<InstalledPluginsViewController> installedPluginsViewControllerProvider,
                          Lazy<MarketplaceViewController> marketplaceViewControllerProvider) {
        super(coreManager, themeManager);
        this.pluginConfigProvider = pluginConfigProvider;
        this.hostServices = hostServicesWrapper.getHostServices();
        this.installedPluginsViewControllerProvider = installedPluginsViewControllerProvider;
        this.marketplaceViewControllerProvider = marketplaceViewControllerProvider;
    }

    public void showAsDialog() {
        Map<Object, Object> properties = new HashMap<>();
        properties.put(WindowProperty.WINDOW_TYPE, WindowType.PLUGIN_SETTINGS);
        if (WindowHelper.focusWindowIfAlreadyThere(properties)) {
            return;
        }
        LoaderResult<PluginsViewController> result = load(PluginsViewController.class, "pluginsView.fxml", () -> this);
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
    private void initialize() {
        setupInstalledPluginTab();
        setupMarketplaceTab();
    }

    private void setupInstalledPluginTab() {
        LoaderResult<InstalledPluginsViewController> result = installedPluginsViewControllerProvider.get().load();
        installedPluginsTab.setContent(result.getMainRegion());
    }

    private void setupMarketplaceTab() {
        LoaderResult<MarketplaceViewController> result = marketplaceViewControllerProvider.get().load();
        marketplaceTab.setContent(result.getMainRegion());
    }

    @FXML
    private void onOpenPluginFolder() {
        hostServices.showDocument(new File(pluginConfigProvider.getPluginPath()).toURI().toString());
    }
}
