package org.correomqtt.gui.controller;

import javafx.application.Platform;
import javafx.collections.FXCollections;
import javafx.fxml.FXML;
import javafx.scene.control.Button;
import javafx.scene.control.Label;
import javafx.scene.control.ListCell;
import javafx.scene.control.ListView;
import javafx.scene.layout.Pane;
import lombok.extern.slf4j.Slf4j;
import org.correomqtt.business.provider.SettingsProvider;
import org.correomqtt.gui.cell.PluginCell;
import org.correomqtt.gui.model.PluginInfoPropertiesDTO;
import org.correomqtt.gui.transformer.PluginTransformer;
import org.correomqtt.gui.utils.PlatformUtils;
import org.correomqtt.plugin.manager.PluginManager;
import org.pf4j.Plugin;

import java.text.MessageFormat;
import java.util.ResourceBundle;

@Slf4j
public class MarketplaceViewController extends BaseController {

    @FXML
    Pane marketplaceRootPane;

    @FXML
    ListView<PluginInfoPropertiesDTO> marketplacePluginList;

    @FXML
    Label pluginName;

    @FXML
    Label pluginDescription;

    @FXML
    Label pluginProvider;

    @FXML
    Label pluginProjectUrl;

    @FXML
    Label pluginRepository;

    @FXML
    Label pluginInstalledVersion;

    @FXML
    Label pluginInstallableVersion;

    @FXML
    Button pluginInstallBtn;

    private final ResourceBundle resources = ResourceBundle.getBundle("org.correomqtt.i18n", SettingsProvider.getInstance().getSettings().getCurrentLocale());

    public static LoaderResult<MarketplaceViewController> load() {
        return load(MarketplaceViewController.class, "marketplaceView.fxml", MarketplaceViewController::new);
    }

    @FXML
    public void initialize() {
        marketplacePluginList.setCellFactory(this::createCell);
        marketplacePluginList.setItems(FXCollections.observableArrayList(
                PluginTransformer.dtoListToPropList(PluginManager.getInstance().getAllPluginsAvailableFromRepos())
        ));
        setCurrentPlugin(null);
    }

    @FXML
    public void onInstall(){
        PluginInfoPropertiesDTO selectedPlugin = marketplacePluginList.getSelectionModel().getSelectedItem();
        PluginManager.getInstance().getUpdateManager().installPlugin(selectedPlugin.getId(),selectedPlugin.getInstallableVersion());
        //TODO async event
        selectedPlugin.getInstallableVersionProperty().set(selectedPlugin.getInstallableVersion());
        Platform.runLater(() -> marketplacePluginList.refresh());
    }

    private ListCell<PluginInfoPropertiesDTO> createCell(ListView<PluginInfoPropertiesDTO> pluginInfoDTOListView) {
        PluginCell cell = new PluginCell(pluginInfoDTOListView);
        cell.selectedProperty().addListener((observable, oldValue, newValue) -> {
            if (Boolean.TRUE.equals(newValue)) {
                setCurrentPlugin(cell.getItem());
            }
        });
        return cell;
    }

    private void setCurrentPlugin(PluginInfoPropertiesDTO plugin) {
        if (plugin == null) {
            pluginName.setText(resources.getString("noPluginSelected"));
            pluginDescription.setVisible(false);
            pluginProvider.setVisible(false);
            pluginProjectUrl.setVisible(false);
            pluginRepository.setVisible(false);
            pluginInstalledVersion.setVisible(false);
            pluginInstallableVersion.setVisible(false);
            pluginInstallBtn.setVisible(false);
            return;
        }

        pluginName.setText(plugin.getName());
        this.setOrHide(pluginDescription, plugin.getDescription());
        this.setOrHide(pluginProvider, plugin.getProvider(), "pluginCreatedBy");
        this.setOrHide(pluginProjectUrl, plugin.getProjectUrl());
        this.setOrHide(pluginRepository, plugin.getRepositoryId(), "pluginRepositoryFrom");
        this.setOrHide(pluginInstalledVersion, plugin.getInstalledVersion(), "pluginInstalledVersion");
        this.setOrHide(pluginInstallableVersion, plugin.getInstallableVersion(), "pluginInstallableVersion");
        pluginInstallBtn.setVisible(plugin.getInstalledVersion() == null);
    }


    private void setOrHide(Label label, String value) {
        setOrHide(label, value, null);
    }

    private void setOrHide(Label label, String value, String messagePattern) {
        if (value == null) {
            label.setManaged(false);
            label.setText(null);
        } else {
            if (messagePattern != null) {
                label.setText(MessageFormat.format(resources.getString(messagePattern), value));
            } else {
                label.setText(value);
            }
            label.setManaged(true);
            label.setVisible(true);
        }
    }

}
