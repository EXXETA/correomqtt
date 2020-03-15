package com.exxeta.correomqtt.gui.controller;

import com.exxeta.correomqtt.business.services.ConfigService;
import com.exxeta.correomqtt.gui.model.WindowProperty;
import com.exxeta.correomqtt.gui.model.WindowType;
import com.exxeta.correomqtt.gui.utils.HostServicesHolder;
import com.exxeta.correomqtt.gui.utils.WindowHelper;
import com.exxeta.correomqtt.plugin.manager.PermissionPlugin;
import com.exxeta.correomqtt.plugin.manager.PluginSecurityPolicy;
import com.exxeta.correomqtt.plugin.manager.PluginSystem;
import javafx.animation.PauseTransition;
import javafx.beans.property.SimpleObjectProperty;
import javafx.collections.FXCollections;
import javafx.collections.ObservableList;
import javafx.fxml.FXML;
import javafx.scene.control.Label;
import javafx.scene.control.*;
import javafx.scene.input.Clipboard;
import javafx.scene.input.ClipboardContent;
import javafx.scene.layout.Pane;
import javafx.scene.paint.Color;
import javafx.util.Duration;
import org.pf4j.Plugin;
import org.pf4j.PluginState;
import org.pf4j.PluginWrapper;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.awt.*;
import java.io.File;
import java.io.IOException;
import java.security.Permission;
import java.security.Permissions;
import java.util.*;

public class PluginsViewController extends BaseController {

    private static final Logger LOGGER = LoggerFactory.getLogger(PluginsViewController.class);

    @FXML
    private Pane pluginRootPane;

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

    private boolean isRestartRequired;

    private static ResourceBundle resources;

    private PluginSystem pluginSystem;

    public static void showAsDialog() {
        Map<Object, Object> properties = new HashMap<>();
        properties.put(WindowProperty.WINDOW_TYPE, WindowType.PLUGIN_SETTINGS);

        if (WindowHelper.focusWindowIfAlreadyThere(properties)) {
            return;
        }

        LoaderResult<PluginsViewController> result = load(PluginsViewController.class, "pluginsView.fxml");
        resources = result.getResourceBundle();
        showAsDialog(result, resources.getString("pluginsViewControllerTitle"), properties, true, false, null, null);
    }

    @FXML
    public void initialize() {
        this.pluginSystem = PluginSystem.getInstance();
        setUpTable();
    }

    private void setUpTable() {
        ArrayList<PluginWrapper> allPluginWrappers = new ArrayList<>(PluginSystem.getInstance().getPlugins());
        ObservableList<PluginWrapper> plugins = FXCollections.observableArrayList(allPluginWrappers);
        pluginsTableView.setItems(plugins);
        isEnabledColumn.setCellValueFactory(this::getIsEnabledCellData);
        isEnabledColumn.setSortable(false);
        nameVersionColumn.setCellValueFactory(this::getPluginNameAndVersionCellData);
        descriptionColumn.setCellValueFactory(cellData -> new SimpleObjectProperty<>(cellData.getValue().getDescriptor().getPluginDescription()));
        providerColumn.setCellValueFactory(cellData -> new SimpleObjectProperty<>(cellData.getValue().getDescriptor().getProvider()));
        permissionColumn.setCellValueFactory(this::getPermissionCellData);
        fileColumn.setCellValueFactory(cellData -> new SimpleObjectProperty<>(cellData.getValue().getPluginPath().toString()));
        pluginsTableView.setRowFactory(this::getRowFactory);
    }

    private SimpleObjectProperty<CheckBox> getIsEnabledCellData(TableColumn.CellDataFeatures<PluginWrapper, CheckBox> cellData) {
        CheckBox checkBox = new CheckBox();
        checkBox.selectedProperty().setValue(!cellData.getValue().getPluginState().equals(PluginState.DISABLED));
        checkBox.selectedProperty().addListener((observableValue, oldValue, newValue) -> {
            if (newValue) {
                pluginSystem.enablePlugin(cellData.getValue().getPluginId());
            } else {
                pluginSystem.disablePlugin(cellData.getValue().getPluginId());
            }
            setStatusRestartRequired();
        });
        return new SimpleObjectProperty<>(checkBox);
    }

    private SimpleObjectProperty<String> getPluginNameAndVersionCellData(TableColumn.CellDataFeatures<PluginWrapper, String> cellData) {
        String name = cellData.getValue().getPluginId();
        String version = cellData.getValue().getDescriptor().getVersion();
        return new SimpleObjectProperty<>(name + " - " + version);
    }

    private SimpleObjectProperty<String> getPermissionCellData(TableColumn.CellDataFeatures<PluginWrapper, String> cellData) {
        StringBuilder permissions = new StringBuilder();
        Plugin plugin = cellData.getValue().getPlugin();
        if (plugin instanceof PermissionPlugin) {
            Permissions pluginPermissions = PluginSecurityPolicy.removeForbiddenPermissions(plugin.getWrapper().getPluginId(),
                    ((PermissionPlugin) plugin).getPermissions());
            Iterator<Permission> it = pluginPermissions.elements().asIterator();
            while (it.hasNext()) {
                Permission p = it.next();
                permissions.append(p.getClass().getSimpleName())
                        .append("(").append(p.getName());
                if (!p.getActions().isBlank()) {
                    permissions.append(", ").append(p.getActions());
                }
                permissions.append("); ");
            }
        }
        return new SimpleObjectProperty<>(permissions.toString());
    }

    private TableRow<PluginWrapper> getRowFactory(TableView<PluginWrapper> tv) {
        TableRow<PluginWrapper> row = new TableRow<>();
        row.setOnMouseClicked(event -> {
            if (event.getClickCount() == 2 && (!row.isEmpty())) {
                PluginWrapper rowData = row.getItem();
                ClipboardContent content = new ClipboardContent();
                content.putString(rowData.getPluginId());
                Clipboard.getSystemClipboard().setContent(content);
                setStatusCopiedToClipboard(rowData.getPluginId());
            }
        });
        return row;
    }

    @FXML
    public void onOpenPluginFolder() {
        HostServicesHolder.getInstance().getHostServices().showDocument(new File(ConfigService.getInstance().getPluginRootPath()).toString());
    }

    private void setStatusRestartRequired() {
        isRestartRequired = true;
        statusText.setText("To apply your changes, please restart the application.");
        statusText.setTextFill(Color.web("ff0000"));
    }

    private void setStatusCopiedToClipboard(String id) {
        statusText.setText("Plugin ID copied to clipboard: " + id);
        statusText.setTextFill(Color.web("000000"));

        PauseTransition wait = new PauseTransition(Duration.seconds(2));
        wait.setOnFinished(e -> {
            if (isRestartRequired) {
                setStatusRestartRequired();
            } else {
                statusText.setText("");
            }
        });
        wait.play();
    }
}
