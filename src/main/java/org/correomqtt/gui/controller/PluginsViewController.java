package org.correomqtt.gui.controller;

import javafx.fxml.FXML;
import javafx.scene.control.Tab;
import org.correomqtt.business.provider.PluginConfigProvider;
import org.correomqtt.gui.model.WindowProperty;
import org.correomqtt.gui.model.WindowType;
import org.correomqtt.gui.utils.HostServicesHolder;
import org.correomqtt.gui.utils.WindowHelper;
import org.correomqtt.plugin.manager.PluginManager;

import java.io.File;
import java.util.HashMap;
import java.util.Map;
import java.util.ResourceBundle;

public class PluginsViewController extends BaseController {

    @FXML
    private Tab marketplaceTab;

    @FXML
    private Tab installedPluginsTab;

    private PluginManager pluginSystem;

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
        this.pluginSystem = PluginManager.getInstance();
        setupInstalledPluginTab();
        setupMarketplaceTab();
    }

    private void setupInstalledPluginTab(){
        LoaderResult<InstalledPluginsViewController> result = InstalledPluginsViewController.load();
        installedPluginsTab.setContent(result.getMainPane());
    }

    private void setupMarketplaceTab(){
        LoaderResult<MarketplaceViewController> result = MarketplaceViewController.load();
        marketplaceTab.setContent(result.getMainPane());
    }

    @FXML
    public void onOpenPluginFolder() {
        HostServicesHolder.getInstance().getHostServices().showDocument(new File(PluginConfigProvider.getInstance().getPluginPath()).toURI().toString());
    }
}
