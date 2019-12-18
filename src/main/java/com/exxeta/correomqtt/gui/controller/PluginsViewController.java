package com.exxeta.correomqtt.gui.controller;

import com.exxeta.correomqtt.gui.model.WindowProperty;
import com.exxeta.correomqtt.gui.model.WindowType;
import com.exxeta.correomqtt.gui.utils.WindowHelper;
import com.exxeta.correomqtt.plugin.manager.PluginSystem;
import javafx.beans.property.SimpleObjectProperty;
import javafx.collections.FXCollections;
import javafx.collections.ObservableList;
import javafx.fxml.FXML;
import javafx.scene.control.CheckBox;
import javafx.scene.control.TableColumn;
import javafx.scene.control.TableRow;
import javafx.scene.control.TableView;
import javafx.scene.input.Clipboard;
import javafx.scene.input.ClipboardContent;
import javafx.scene.layout.Pane;
import javafx.stage.Stage;
import org.pf4j.PluginState;
import org.pf4j.PluginWrapper;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.Map;
import java.util.ResourceBundle;

public class PluginsViewController extends BaseController {

    private static final Logger LOGGER = LoggerFactory.getLogger(PluginsViewController.class);

    @FXML
    private Pane pluginRootPane;

    @FXML
    private TableView<PluginWrapper> pluginsTableView;

    @FXML
    private TableColumn<PluginWrapper, CheckBox> isEnabledColumn;

    @FXML
    private TableColumn<PluginWrapper, String> nameColumn;

    @FXML
    private TableColumn<PluginWrapper, String> versionColumn;

    @FXML
    private TableColumn<PluginWrapper, String> providerColumn;

    @FXML
    private TableColumn<PluginWrapper, String> fileColumn;

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
        isEnabledColumn.setCellValueFactory(cellData -> {
            CheckBox checkBox = new CheckBox();
            checkBox.selectedProperty().setValue(!cellData.getValue().getPluginState().equals(PluginState.DISABLED));
            checkBox.selectedProperty().addListener((observableValue, oldValue, newValue) -> {
                if (newValue) {
                    pluginSystem.enablePlugin(cellData.getValue().getPluginId());
                    pluginSystem.startPlugin(cellData.getValue().getPluginId());
                } else {
                    pluginSystem.disablePlugin(cellData.getValue().getPluginId());
                }
            });
            return new SimpleObjectProperty<>(checkBox);
        });
        isEnabledColumn.setSortable(false);
        nameColumn.setCellValueFactory(cellData -> new SimpleObjectProperty<>(cellData.getValue().getPluginId()));
        versionColumn.setCellValueFactory(cellData -> new SimpleObjectProperty<>(cellData.getValue().getDescriptor().getVersion()));
        providerColumn.setCellValueFactory(cellData -> new SimpleObjectProperty<>(cellData.getValue().getDescriptor().getProvider()));
        fileColumn.setCellValueFactory(cellData -> new SimpleObjectProperty<>(cellData.getValue().getPluginPath().toString()));
        pluginsTableView.setRowFactory(tv -> {
            TableRow<PluginWrapper> row = new TableRow<>();
            row.setOnMouseClicked(event -> {
                if (event.getClickCount() == 2 && (!row.isEmpty())) {
                    PluginWrapper rowData = row.getItem();
                    ClipboardContent content = new ClipboardContent();
                    content.putString(rowData.getPluginId());
                    Clipboard.getSystemClipboard().setContent(content);
                }
            });
            return row;
        });
    }

    @FXML
    public void onSaveClicked() {
        LOGGER.debug("Save in plugins clicked");
        closeDialog();
    }

    private void closeDialog() {
        Stage stage = (Stage) pluginRootPane.getScene().getWindow();
        stage.close();
    }
}
