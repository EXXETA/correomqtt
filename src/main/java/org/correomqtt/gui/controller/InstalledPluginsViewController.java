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
import org.correomqtt.business.dispatcher.PluginDisableDispatcher;
import org.correomqtt.business.dispatcher.PluginDisableObserver;
import org.correomqtt.business.dispatcher.PluginEnableDispatcher;
import org.correomqtt.business.dispatcher.PluginEnableObserver;
import org.correomqtt.business.dispatcher.PluginInstallDispatcher;
import org.correomqtt.business.dispatcher.PluginInstallObserver;
import org.correomqtt.business.dispatcher.PluginUninstallDispatcher;
import org.correomqtt.business.dispatcher.PluginUninstallObserver;
import org.correomqtt.business.provider.SettingsProvider;
import org.correomqtt.gui.business.PluginTaskFactory;
import org.correomqtt.gui.cell.PluginCell;
import org.correomqtt.gui.helper.AlertHelper;
import org.correomqtt.gui.model.PluginInfoPropertiesDTO;
import org.correomqtt.gui.transformer.PluginTransformer;
import org.correomqtt.plugin.manager.PluginManager;

import java.text.MessageFormat;
import java.util.ResourceBundle;

@Slf4j
public class InstalledPluginsViewController extends BaseControllerImpl implements
        PluginInstallObserver,
        PluginUninstallObserver,
        PluginDisableObserver,
        PluginEnableObserver {

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
        PluginInstallDispatcher.getInstance().addObserver(this);
        PluginUninstallDispatcher.getInstance().addObserver(this);
        PluginDisableDispatcher.getInstance().addObserver(this);
        PluginEnableDispatcher.getInstance().addObserver(this);
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
            PluginTaskFactory.enable(selectedPlugin.getId());
        } else {
            PluginTaskFactory.disable(selectedPlugin.getId());
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
            PluginTaskFactory.uninstall(selectedPlugin.getId());
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

    @Override
    public void onPluginInstallSucceeded(String pluginId, String version) {
        reloadData(pluginId);
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

    @Override
    public void onPluginInstallCancelled(String pluginId, String version) {
        Platform.runLater(() -> installedPluginsRootPane.setDisable(false));
    }

    private void showFail() {
        AlertHelper.warn(resources.getString("pluginOperationFailedTitle"), resources.getString("pluginOperationFailedContent"), true);
        Platform.runLater(() -> installedPluginsRootPane.setDisable(false));
    }

    @Override
    public void onPluginInstallFailed(String pluginId, String version, Throwable exception) {
        Platform.runLater(() -> installedPluginsRootPane.setDisable(false));
    }

    @Override
    public void onPluginInstallStarted(String pluginId, String version) {
        installedPluginsRootPane.setDisable(true);
    }

    @Override
    public void onPluginUninstallSucceeded(String pluginId) {
        reloadData(pluginId);
        Platform.runLater(() -> AlertHelper.info(resources.getString("pluginChangeTitle"),
                resources.getString("pluginChangeContent")));
    }

    @Override
    public void onPluginUninstallCancelled(String pluginId) {
        showFail();
    }

    @Override
    public void onPluginUninstallFailed(String pluginId, Throwable exception) {
        showFail();
    }

    @Override
    public void onPluginUninstallStarted(String pluginId) {
        installedPluginsRootPane.setDisable(true);
    }

    @Override
    public void onPluginDisableSucceeded(String pluginId) {
        this.onPluginUninstallSucceeded(pluginId);
    }

    @Override
    public void onPluginDisableCancelled(String pluginId) {
        showFail();
    }

    @Override
    public void onPluginDisableFailed(String pluginId, Throwable exception) {
        showFail();
    }

    @Override
    public void onPluginDisableStarted(String pluginId) {
        installedPluginsRootPane.setDisable(true);
    }

    @Override
    public void onPluginEnableSucceeded(String pluginId) {
        reloadData(pluginId);
        Platform.runLater(() -> AlertHelper.info(resources.getString("pluginChangeTitle"),
                resources.getString("pluginChangeContent")));
    }

    @Override
    public void onPluginEnableCancelled(String pluginId) {
        showFail();
    }

    @Override
    public void onPluginEnableFailed(String pluginId, Throwable exception) {
        showFail();
    }

    @Override
    public void onPluginEnableStarted(String pluginId) {
        installedPluginsRootPane.setDisable(true);
    }
}
