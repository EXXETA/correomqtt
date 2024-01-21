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
import org.correomqtt.core.eventbus.Subscribe;
import org.correomqtt.core.plugin.marketplace.PluginDisabledEvent;
import org.correomqtt.core.plugin.marketplace.PluginDisabledFailedEvent;
import org.correomqtt.core.plugin.marketplace.PluginDisabledStartedEvent;
import org.correomqtt.core.plugin.marketplace.PluginEnabledEvent;
import org.correomqtt.core.plugin.marketplace.PluginEnabledFailedEvent;
import org.correomqtt.core.plugin.marketplace.PluginEnabledStartedEvent;
import org.correomqtt.core.plugin.marketplace.PluginInstallEvent;
import org.correomqtt.core.plugin.marketplace.PluginInstallFailedEvent;
import org.correomqtt.core.plugin.marketplace.PluginInstallStartedEvent;
import org.correomqtt.core.plugin.marketplace.PluginInstallTask;
import org.correomqtt.core.plugin.marketplace.PluginUninstallEvent;
import org.correomqtt.core.plugin.marketplace.PluginUninstallFailedEvent;
import org.correomqtt.core.plugin.marketplace.PluginUninstallStartedEvent;
import org.correomqtt.gui.model.PluginInfoPropertiesDTO;
import org.correomqtt.gui.theme.ThemeManager;
import org.correomqtt.gui.transformer.PluginTransformer;
import org.correomqtt.gui.utils.AlertHelper;
import org.correomqtt.gui.views.LoaderResult;
import org.correomqtt.gui.views.base.BaseControllerImpl;

import javax.inject.Inject;
import java.text.MessageFormat;
import java.util.ResourceBundle;

@Slf4j
public class MarketplaceViewController extends BaseControllerImpl {

    private final PluginInstallTask.Factory pluginInstallTaskFactory;
    private final ResourceBundle resources;
    private final PluginCell.Factory pluginCellFactory;
    private final AlertHelper alertHelper;
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
    @FXML
    Label pluginUpdateLabel;

    @Inject
    public MarketplaceViewController(CoreManager coreManager,
                                     ThemeManager themeManager,
                                     PluginInstallTask.Factory pluginInstallTaskFactory,
                                     PluginCell.Factory pluginCellFactory,
                                     AlertHelper alertHelper) {
        super(coreManager, themeManager);
        this.pluginInstallTaskFactory = pluginInstallTaskFactory;
        resources = ResourceBundle.getBundle("org.correomqtt.i18n", coreManager.getSettingsManager().getSettings().getCurrentLocale());

        this.pluginCellFactory = pluginCellFactory;
        this.alertHelper = alertHelper;
    }

    public LoaderResult<MarketplaceViewController> load() {
        return load(MarketplaceViewController.class, "marketplaceView.fxml", () -> this);
    }

    @FXML
    private void initialize() {
        marketplacePluginList.setCellFactory(this::createCell);
        marketplacePluginList.setItems(FXCollections.observableArrayList(
                PluginTransformer.dtoListToPropList(coreManager.getPluginManager().getAllPluginsAvailableFromRepos())
        ));
        setCurrentPlugin(null);
    }

    @FXML
    private void onInstall() {
        PluginInfoPropertiesDTO selectedPlugin = marketplacePluginList.getSelectionModel().getSelectedItem();
        pluginInstallTaskFactory.create(selectedPlugin.getId(), selectedPlugin.getInstallableVersion());
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

    @SuppressWarnings("unused")
    public void onPluginInstallSucceeded(@Subscribe PluginInstallEvent event) {
        reloadData(event.pluginId());
        Platform.runLater(() -> alertHelper.info(resources.getString("pluginChangeTitle"),
                resources.getString("pluginChangeContent")));
    }

    private void reloadData(String pluginId) {
        marketplacePluginList.setItems(FXCollections.observableArrayList(
                PluginTransformer.dtoListToPropList(coreManager.getPluginManager().getAllPluginsAvailableFromRepos())
        ));
        Platform.runLater(() -> {
            marketplaceRootPane.setDisable(false);
            marketplacePluginList.refresh();
            PluginInfoPropertiesDTO currentPlugin = marketplacePluginList.getItems().stream()
                    .filter(p -> p.getId().equals(pluginId))
                    .findFirst()
                    .orElse(null);
            setCurrentPlugin(currentPlugin);
            marketplacePluginList.getSelectionModel().select(currentPlugin);
        });
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
        pluginUpdateLabel.setVisible(plugin.getInstalledVersion() != null &&
                plugin.getInstallableVersion() != null &&
                plugin.getInstalledVersion().compareTo(plugin.getInstallableVersion()) < 0);
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

    @SuppressWarnings("unused")
    @Subscribe(PluginInstallFailedEvent.class)
    public void onPluginInstallFailed() {
        showFail();
    }

    private void showFail() {
        alertHelper.warn(resources.getString("pluginOperationFailedTitle"), resources.getString("pluginOperationFailedContent"), true);
        Platform.runLater(() -> marketplaceRootPane.setDisable(false));
    }

    @SuppressWarnings("unused")
    @Subscribe(PluginInstallStartedEvent.class)
    public void onPluginInstallStarted() {
        marketplaceRootPane.setDisable(true);
    }


    @SuppressWarnings("unused")
    public void onPluginUninstallSucceeded(@Subscribe PluginUninstallEvent event) {
        reloadData(event.pluginId());
    }

    @SuppressWarnings("unused")
    @Subscribe(PluginUninstallFailedEvent.class)
    public void onPluginUninstallFailed() {
        Platform.runLater(() -> marketplaceRootPane.setDisable(false));
    }

    @SuppressWarnings("unused")
    @Subscribe(PluginUninstallStartedEvent.class)
    public void onPluginUninstallStarted() {
        marketplaceRootPane.setDisable(true);
    }

    @SuppressWarnings("unused")
    @Subscribe(PluginDisabledStartedEvent.class)
    public void onPluginDisableStarted() {
        marketplaceRootPane.setDisable(true);
    }

    @SuppressWarnings("unused")
    public void onPluginDisableSucceeded(@Subscribe PluginDisabledEvent event) {
        reloadData(event.pluginId());
    }

    @SuppressWarnings("unused")
    @Subscribe(PluginDisabledFailedEvent.class)
    public void onPluginDisableFailed() {
        Platform.runLater(() -> marketplaceRootPane.setDisable(false));
    }

    @SuppressWarnings("unused")
    @Subscribe(PluginEnabledStartedEvent.class)
    public void onPluginEnableStarted() {
        marketplaceRootPane.setDisable(true);
    }


    @SuppressWarnings("unused")
    public void onPluginEnableSucceeded(@Subscribe PluginEnabledEvent event) {
        reloadData(event.pluginId());
    }

    @SuppressWarnings("unused")
    @Subscribe(PluginEnabledFailedEvent.class)
    public void onPluginEnableFailed() {
        Platform.runLater(() -> marketplaceRootPane.setDisable(false));
    }
}
