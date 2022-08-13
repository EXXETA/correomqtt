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
import org.correomqtt.gui.helper.AlertHelper;
import org.correomqtt.gui.model.PluginInfoPropertiesDTO;
import org.correomqtt.gui.transformer.PluginTransformer;
import org.correomqtt.plugin.manager.PluginManager;

import java.text.MessageFormat;
import java.util.ResourceBundle;

@Slf4j
public class InstalledPluginsViewController extends BaseController {

    @FXML
    Pane installedPluginsRootPane;

    @FXML
    ListView<PluginInfoPropertiesDTO> installedPluginList;

    @FXML
    Label pluginName;

    @FXML
    Label pluginDescription;

    @FXML
    Label pluginProvider;

    @FXML
    Label pluginLicense;

    @FXML
    Label pluginInstalledVersion;

    @FXML
    Label pluginPath;


    @FXML
    Button pluginDisableToggleBtn;

    @FXML
    Button pluginUninstallBtn;

    private final ResourceBundle resources = ResourceBundle.getBundle("org.correomqtt.i18n", SettingsProvider.getInstance().getSettings().getCurrentLocale());

    public static LoaderResult<InstalledPluginsViewController> load() {
        return load(InstalledPluginsViewController.class, "installedPluginsView.fxml", InstalledPluginsViewController::new);
    }

    @FXML
    public void initialize() {
        installedPluginList.setCellFactory(this::createCell);
        installedPluginList.setItems(FXCollections.observableArrayList(
                PluginTransformer.dtoListToPropList(PluginManager.getInstance().getInstalledPlugins())
        ));
        setCurrentPlugin(null);
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
            pluginInstalledVersion.setVisible(false);
            pluginLicense.setVisible(false);
            pluginDisableToggleBtn.setVisible(false);
            pluginUninstallBtn.setVisible(false);
            pluginPath.setVisible(false);
            return;
        }

        pluginName.setText(plugin.getName());
        this.setOrHide(pluginDescription, plugin.getDescription());
        this.setOrHide(pluginProvider, plugin.getProvider(), "pluginCreatedBy");
        this.setOrHide(pluginLicense, plugin.getLicense(), "pluginLicense");
        this.setOrHide(pluginInstalledVersion, plugin.getInstalledVersion(), "pluginInstalledVersion");
        this.setOrHide(pluginPath, plugin.getPath().toString(),"pluginPath");
        pluginDisableToggleBtn.setVisible(true);
        pluginDisableToggleBtn.setText(resources.getString(Boolean.TRUE.equals(plugin.getDisabled()) ? "enable" : "disable"));
        pluginUninstallBtn.setVisible(true);
    }


    @FXML
    public void onDisableToggle() {
        PluginInfoPropertiesDTO selectedPlugin = installedPluginList.getSelectionModel().getSelectedItem();
        if (Boolean.TRUE.equals(selectedPlugin.getDisabled())) {
            log.info("Enable plugin " + selectedPlugin.getId());
            PluginManager.getInstance().enablePlugin(selectedPlugin.getId());
            selectedPlugin.getDisabledProperty().set(false);
            installedPluginList.refresh();
        } else {
            log.info("Disable plugin " + selectedPlugin.getId());
            PluginManager.getInstance().disablePlugin(selectedPlugin.getId());
            selectedPlugin.getDisabledProperty().set(true);
        }
        Platform.runLater(() -> installedPluginList.refresh());
        //TODO inform user about required restart
    }

    @FXML
    public void onUninstall() {
        PluginInfoPropertiesDTO selectedPlugin = installedPluginList.getSelectionModel().getSelectedItem();
        boolean confirmed = AlertHelper.confirm(
                resources.getString("reallyUninstallTitle"),
                MessageFormat.format(resources.getString("reallyUninstallHeader"), selectedPlugin.getName(), selectedPlugin.getInstalledVersion()),
                MessageFormat.format(resources.getString("reallyUninstallContent"), selectedPlugin.getName(), selectedPlugin.getInstalledVersion()),
                resources.getString("commonCancelButton"),
                resources.getString("reallyUninstallYesButton"));
        if (confirmed) {
            installedPluginList.getItems().remove(selectedPlugin);

            //TODO async event
            PluginManager.getInstance().deletePlugin(selectedPlugin.getId());
            installedPluginList.getSelectionModel().clearSelection();
            setCurrentPlugin(null);
            //TODO inform user about required restart
        }
    }


    private void setOrHide(Label label, String value) {
        setOrHide(label, value, null);
    }

    private void setOrHide(Label label, String value, String messagePattern) {
        if (value == null || value.isEmpty()) {
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
