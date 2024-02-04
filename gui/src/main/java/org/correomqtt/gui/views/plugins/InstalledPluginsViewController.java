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
import org.correomqtt.core.CoreManager;
import org.correomqtt.core.plugin.marketplace.PluginDisableTaskFactory;
import org.correomqtt.core.plugin.marketplace.PluginDisabledEvent;
import org.correomqtt.core.plugin.marketplace.PluginDisabledFailedEvent;
import org.correomqtt.core.plugin.marketplace.PluginDisabledStartedEvent;
import org.correomqtt.core.plugin.marketplace.PluginEnableTaskFactory;
import org.correomqtt.core.plugin.marketplace.PluginEnabledEvent;
import org.correomqtt.core.plugin.marketplace.PluginEnabledFailedEvent;
import org.correomqtt.core.plugin.marketplace.PluginEnabledStartedEvent;
import org.correomqtt.core.plugin.marketplace.PluginInstallEvent;
import org.correomqtt.core.plugin.marketplace.PluginInstallFailedEvent;
import org.correomqtt.core.plugin.marketplace.PluginInstallStartedEvent;
import org.correomqtt.core.plugin.marketplace.PluginUninstallEvent;
import org.correomqtt.core.plugin.marketplace.PluginUninstallFailedEvent;
import org.correomqtt.core.plugin.marketplace.PluginUninstallTaskFactory;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import org.correomqtt.di.Observes;
import org.correomqtt.gui.model.PluginInfoPropertiesDTO;
import org.correomqtt.gui.theme.ThemeManager;
import org.correomqtt.gui.transformer.PluginTransformer;
import org.correomqtt.gui.utils.AlertHelper;
import org.correomqtt.gui.views.LoaderResult;
import org.correomqtt.gui.views.base.BaseControllerImpl;

import java.text.MessageFormat;
import java.util.ResourceBundle;

@Slf4j
@DefaultBean
public class InstalledPluginsViewController extends BaseControllerImpl {

    private final PluginEnableTaskFactory pluginEnableTaskFactory;
    private final PluginDisableTaskFactory pluginDisableTaskFactory;
    private final PluginUninstallTaskFactory pluginUninstallTaskFactory;
    private final PluginCellFactory pluginCellFactory;
    private final ResourceBundle resources;
    private final AlertHelper alertHelper;
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

    @Inject
    public InstalledPluginsViewController(CoreManager coreManager,
                                          PluginEnableTaskFactory pluginEnableTaskFactory,
                                          PluginDisableTaskFactory pluginDisableTaskFactory,
                                          PluginUninstallTaskFactory pluginUninstallTaskFactory,
                                          PluginCellFactory pluginCellFactory,
                                          ThemeManager themeManager,
                                          AlertHelper alertHelper) {
        super(coreManager, themeManager);
        this.pluginEnableTaskFactory = pluginEnableTaskFactory;
        this.pluginDisableTaskFactory = pluginDisableTaskFactory;
        this.pluginUninstallTaskFactory = pluginUninstallTaskFactory;
        this.pluginCellFactory = pluginCellFactory;
        resources = ResourceBundle.getBundle("org.correomqtt.i18n", coreManager.getSettingsManager().getSettings().getCurrentLocale());
        this.alertHelper = alertHelper;
    }

    public LoaderResult<InstalledPluginsViewController> load() {
        return load(InstalledPluginsViewController.class, "installedPluginsView.fxml", () -> this);
    }

    @FXML
    private void initialize() {
        installedPluginList.setCellFactory(this::createCell);
        installedPluginList.setItems(FXCollections.observableArrayList(
                PluginTransformer.dtoListToPropList(coreManager.getPluginManager().getInstalledPlugins())
        ));
        setCurrentPlugin(null);
    }


    private ListCell<PluginInfoPropertiesDTO> createCell(ListView<PluginInfoPropertiesDTO> pluginInfoDTOListView) {
        PluginCell cell = pluginCellFactory.create(pluginInfoDTOListView);
        cell.selectedProperty().addListener((observable, oldValue, newValue) -> {
            if (Boolean.TRUE.equals(newValue)) {
                setCurrentPlugin(cell.getItem());
            }
        });
        return cell;
    }

    @FXML
    private void onDisableToggle() {
        PluginInfoPropertiesDTO selectedPlugin = installedPluginList.getSelectionModel().getSelectedItem();
        if (Boolean.TRUE.equals(selectedPlugin.getDisabled())) {
            pluginEnableTaskFactory.create(selectedPlugin.getId()).run();
        } else {
            pluginDisableTaskFactory.create(selectedPlugin.getId()).run();
        }
        Platform.runLater(() -> installedPluginList.refresh());
    }

    @FXML
    private void onUninstall() {
        PluginInfoPropertiesDTO selectedPlugin = installedPluginList.getSelectionModel().getSelectedItem();
        if (alertHelper.confirm(
                resources.getString("reallyUninstallTitle"),
                MessageFormat.format(resources.getString("reallyUninstallHeader"), selectedPlugin.getName(), selectedPlugin.getInstalledVersion()),
                MessageFormat.format(resources.getString("reallyUninstallContent"), selectedPlugin.getName(), selectedPlugin.getInstalledVersion()),
                resources.getString("commonCancelButton"),
                resources.getString("reallyUninstallYesButton"))) {
            pluginUninstallTaskFactory.create(selectedPlugin.getId()).run();
        }
    }

    @SuppressWarnings("unused")
    public void onPluginInstallSucceeded(@Observes PluginInstallEvent event) {
        reloadData(event.pluginId());
    }

    private void reloadData(String pluginId) {
        installedPluginList.setItems(FXCollections.observableArrayList(
                PluginTransformer.dtoListToPropList(coreManager.getPluginManager().getInstalledPlugins())
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
    @Observes(PluginInstallFailedEvent.class)
    public void onPluginInstallFailed() {
        Platform.runLater(() -> installedPluginsRootPane.setDisable(false));
    }

    @SuppressWarnings("unused")
    @Observes(PluginInstallStartedEvent.class)
    public void onPluginInstallStarted() {
        installedPluginsRootPane.setDisable(true);
    }

    public void onPluginUninstallSucceeded(@Observes PluginUninstallEvent event) {
        reloadData(event.pluginId());
        Platform.runLater(() -> alertHelper.info(resources.getString("pluginChangeTitle"),
                resources.getString("pluginChangeContent")));
    }

    @SuppressWarnings("unused")
    @Observes(PluginUninstallFailedEvent.class)
    public void onPluginUninstallFailed() {
        showFail();
    }

    private void showFail() {
        alertHelper.warn(resources.getString("pluginOperationFailedTitle"), resources.getString("pluginOperationFailedContent"), true);
        Platform.runLater(() -> installedPluginsRootPane.setDisable(false));
    }

    @SuppressWarnings("unused")
    @Observes(PluginInstallStartedEvent.class)
    public void onPluginUninstallStarted() {
        installedPluginsRootPane.setDisable(true);
    }

    @SuppressWarnings("unused")
    public void onPluginDisableSucceeded(@Observes PluginDisabledEvent event) {
        //TODO?
    }

    @SuppressWarnings("unused")
    @Observes(PluginDisabledFailedEvent.class)
    public void onPluginDisableFailed() {
        showFail();
    }

    @SuppressWarnings("unused")
    @Observes(PluginDisabledStartedEvent.class)
    public void onPluginDisableStarted() {
        installedPluginsRootPane.setDisable(true);
    }

    @SuppressWarnings("unused")
    public void onPluginEnableSucceeded(@Observes PluginEnabledEvent event) {
        reloadData(event.pluginId());
        Platform.runLater(() -> alertHelper.info(resources.getString("pluginChangeTitle"),
                resources.getString("pluginChangeContent")));
    }

    @SuppressWarnings("unused")
    @Observes(PluginEnabledFailedEvent.class)
    public void onPluginEnableFailed() {
        showFail();
    }

    @SuppressWarnings("unused")
    @Observes(PluginEnabledStartedEvent.class)
    public void onPluginEnableStarted() {
        installedPluginsRootPane.setDisable(true);
    }
}
