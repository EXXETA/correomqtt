package org.correomqtt.gui.views.plugins;

import javafx.application.Platform;
import javafx.collections.FXCollections;
import javafx.fxml.FXML;
import javafx.scene.control.Button;
import javafx.scene.control.Label;
import javafx.scene.control.ListCell;
import javafx.scene.control.ListView;
import javafx.scene.layout.Pane;
import lombok.extern.slf4j.Slf4j;
import org.correomqtt.business.eventbus.EventBus;
import org.correomqtt.business.eventbus.Subscribe;
import org.correomqtt.business.fileprovider.SettingsProvider;
import org.correomqtt.business.plugin.PluginDisableTask;
import org.correomqtt.business.plugin.PluginDisabledEvent;
import org.correomqtt.business.plugin.PluginDisabledFailedEvent;
import org.correomqtt.business.plugin.PluginDisabledStartedEvent;
import org.correomqtt.business.plugin.PluginEnableTask;
import org.correomqtt.business.plugin.PluginEnabledEvent;
import org.correomqtt.business.plugin.PluginEnabledFailedEvent;
import org.correomqtt.business.plugin.PluginEnabledStartedEvent;
import org.correomqtt.business.plugin.PluginInstallEvent;
import org.correomqtt.business.plugin.PluginInstallFailedEvent;
import org.correomqtt.business.plugin.PluginInstallStartedEvent;
import org.correomqtt.business.plugin.PluginUninstallEvent;
import org.correomqtt.business.plugin.PluginUninstallFailedEvent;
import org.correomqtt.business.plugin.PluginUninstallTask;
import org.correomqtt.gui.views.base.BaseControllerImpl;
import org.correomqtt.gui.views.LoaderResult;
import org.correomqtt.gui.utils.AlertHelper;
import org.correomqtt.gui.model.PluginInfoPropertiesDTO;
import org.correomqtt.gui.transformer.PluginTransformer;
import org.correomqtt.plugin.manager.PluginManager;

import java.text.MessageFormat;
import java.util.ResourceBundle;

@Slf4j
public class InstalledPluginsViewController extends BaseControllerImpl {

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
    Label pluginBundledLabel;

    @FXML
    Button pluginUninstallBtn;

    private final ResourceBundle resources = ResourceBundle.getBundle("org.correomqtt.i18n", SettingsProvider.getInstance().getSettings().getCurrentLocale());

    public InstalledPluginsViewController() {
        super();
        EventBus.register(this);
    }

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
            pluginBundledLabel.setVisible(false);
            pluginPath.setVisible(false);
            return;
        }

        pluginName.setText(plugin.getName());
        this.setOrHide(pluginDescription, plugin.getDescription());
        this.setOrHide(pluginProvider, plugin.getProvider(), "pluginCreatedBy");
        this.setOrHide(pluginLicense, plugin.getLicense(), "pluginLicense");
        this.setOrHide(pluginInstalledVersion, plugin.getInstalledVersion(), "pluginInstalledVersion");
        this.setOrHide(pluginPath, plugin.getPath().toString(), "pluginPath");
        pluginDisableToggleBtn.setVisible(true);
        pluginDisableToggleBtn.setText(resources.getString(Boolean.TRUE.equals(plugin.getDisabled()) ? "enable" : "disable"));
        Boolean bundled = plugin.getBundled();
        pluginUninstallBtn.setVisible(!bundled);
        pluginUninstallBtn.setManaged(!bundled);
        pluginBundledLabel.setVisible(bundled);
        pluginBundledLabel.setManaged(bundled);

    }


    @FXML
    public void onDisableToggle() {
        PluginInfoPropertiesDTO selectedPlugin = installedPluginList.getSelectionModel().getSelectedItem();
        if (Boolean.TRUE.equals(selectedPlugin.getDisabled())) {
            new PluginEnableTask(selectedPlugin.getId())
                    .run();
        } else {
            new PluginDisableTask(selectedPlugin.getId())
                    .run();
        }
        Platform.runLater(() -> installedPluginList.refresh());
    }

    @FXML
    public void onUninstall() {
        PluginInfoPropertiesDTO selectedPlugin = installedPluginList.getSelectionModel().getSelectedItem();
        if (AlertHelper.confirm(
                resources.getString("reallyUninstallTitle"),
                MessageFormat.format(resources.getString("reallyUninstallHeader"), selectedPlugin.getName(), selectedPlugin.getInstalledVersion()),
                MessageFormat.format(resources.getString("reallyUninstallContent"), selectedPlugin.getName(), selectedPlugin.getInstalledVersion()),
                resources.getString("commonCancelButton"),
                resources.getString("reallyUninstallYesButton"))) {
            new PluginUninstallTask(selectedPlugin.getId()).run();
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

    @SuppressWarnings("unused")
    public void onPluginInstallSucceeded(@Subscribe PluginInstallEvent event) {
        reloadData(event.pluginId());
    }

    private void reloadData(String pluginId) {
        installedPluginList.setItems(FXCollections.observableArrayList(
                PluginTransformer.dtoListToPropList(PluginManager.getInstance().getInstalledPlugins())
        ));
        Platform.runLater(() -> {
            installedPluginsRootPane.setDisable(false);
            installedPluginList.refresh();
            PluginInfoPropertiesDTO currentPlugin = installedPluginList.getItems().stream()
                    .filter(p -> p.getId().equals(pluginId))
                    .findFirst()
                    .orElse(null);
            setCurrentPlugin(currentPlugin);
            installedPluginList.getSelectionModel().select(currentPlugin);
        });
    }

    private void showFail() {
        AlertHelper.warn(resources.getString("pluginOperationFailedTitle"), resources.getString("pluginOperationFailedContent"), true);
        Platform.runLater(() -> installedPluginsRootPane.setDisable(false));
    }

    @SuppressWarnings("unused")
    @Subscribe(PluginInstallFailedEvent.class)
    public void onPluginInstallFailed() {
        Platform.runLater(() -> installedPluginsRootPane.setDisable(false));
    }

    @SuppressWarnings("unused")
    @Subscribe(PluginInstallStartedEvent.class)
    public void onPluginInstallStarted() {
        installedPluginsRootPane.setDisable(true);
    }

    public void onPluginUninstallSucceeded(@Subscribe PluginUninstallEvent event) {
        reloadData(event.pluginId());
        Platform.runLater(() -> AlertHelper.info(resources.getString("pluginChangeTitle"),
                resources.getString("pluginChangeContent")));
    }

    @SuppressWarnings("unused")
    @Subscribe(PluginUninstallFailedEvent.class)
    public void onPluginUninstallFailed() {
        showFail();
    }

    @SuppressWarnings("unused")
    @Subscribe(PluginInstallStartedEvent.class)
    public void onPluginUninstallStarted() {
        installedPluginsRootPane.setDisable(true);
    }

    @SuppressWarnings("unused")
    public void onPluginDisableSucceeded(@Subscribe PluginDisabledEvent event) {
        //TODO?
    }

    @SuppressWarnings("unused")
    @Subscribe(PluginDisabledFailedEvent.class)
    public void onPluginDisableFailed() {
        showFail();
    }

    @SuppressWarnings("unused")
    @Subscribe(PluginDisabledStartedEvent.class)
    public void onPluginDisableStarted() {
        installedPluginsRootPane.setDisable(true);
    }

    @SuppressWarnings("unused")
    public void onPluginEnableSucceeded(@Subscribe PluginEnabledEvent event) {
        reloadData(event.pluginId());
        Platform.runLater(() -> AlertHelper.info(resources.getString("pluginChangeTitle"),
                resources.getString("pluginChangeContent")));
    }

    @SuppressWarnings("unused")
    @Subscribe(PluginEnabledFailedEvent.class)
    public void onPluginEnableFailed() {
        showFail();
    }

    @SuppressWarnings("unused")
    @Subscribe(PluginEnabledStartedEvent.class)
    public void onPluginEnableStarted() {
        installedPluginsRootPane.setDisable(true);
    }
}
