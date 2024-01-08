package org.correomqtt.gui.views.scripting;

import javafx.collections.FXCollections;
import javafx.collections.ListChangeListener;
import javafx.collections.ObservableList;
import javafx.event.ActionEvent;
import javafx.fxml.FXML;
import javafx.scene.control.Button;
import javafx.scene.control.ComboBox;
import javafx.scene.control.Label;
import javafx.scene.control.ListCell;
import javafx.scene.control.ListView;
import javafx.scene.control.SplitPane;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.Pane;
import javafx.scene.layout.Region;
import javafx.stage.WindowEvent;
import lombok.AllArgsConstructor;
import org.correomqtt.business.concurrent.ErrorListener;
import org.correomqtt.business.connection.ConnectionStateChangedEvent;
import org.correomqtt.business.eventbus.EventBus;
import org.correomqtt.business.eventbus.Subscribe;
import org.correomqtt.business.fileprovider.ScriptingProvider;
import org.correomqtt.business.fileprovider.SettingsProvider;
import org.correomqtt.business.mqtt.CorreoMqttClient;
import org.correomqtt.business.connection.ConnectionState;
import org.correomqtt.business.scripting.ScriptDeleteTask;
import org.correomqtt.business.scripting.ScriptExecutionFailedEvent;
import org.correomqtt.business.scripting.ScriptExecutionProgressEvent;
import org.correomqtt.business.scripting.ScriptExecutionSuccessEvent;
import org.correomqtt.business.scripting.ScriptFileDTO;
import org.correomqtt.business.scripting.ScriptLoadTask;
import org.correomqtt.business.scripting.ScriptNewTask;
import org.correomqtt.business.scripting.ScriptRenameTask;
import org.correomqtt.business.utils.ConnectionHolder;
import org.correomqtt.gui.controls.IconButton;
import org.correomqtt.gui.model.ConnectionPropertiesDTO;
import org.correomqtt.gui.model.WindowProperty;
import org.correomqtt.gui.model.WindowType;
import org.correomqtt.gui.transformer.ConnectionTransformer;
import org.correomqtt.gui.utils.AlertHelper;
import org.correomqtt.gui.utils.WindowHelper;
import org.correomqtt.gui.views.LoaderResult;
import org.correomqtt.gui.views.base.BaseControllerImpl;
import org.correomqtt.gui.views.cell.ConnectionCell;
import org.fxmisc.flowless.VirtualizedScrollPane;
import org.fxmisc.richtext.CodeArea;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;
import java.nio.file.Path;
import java.text.MessageFormat;
import java.time.Duration;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.ResourceBundle;

public class ScriptingViewController extends BaseControllerImpl implements ScriptContextMenuDelegate, SingleEditorViewDelegate {

    private static final Logger LOGGER = LoggerFactory.getLogger(ScriptingViewController.class);


    @AllArgsConstructor
    private static class ScriptEditorState {
        private SingleEditorViewController controller;
        private Region region;
    }

    private final HashMap<String, ScriptEditorState> editorStates = new HashMap<>();
    @FXML
    public AnchorPane executionHolder;
    @FXML
    public SplitPane mainSplitPane;
    @FXML
    public AnchorPane scriptListSidebar;
    @FXML
    public AnchorPane editorPane;
    @FXML
    public SplitPane scriptSplitPane;
    @FXML
    public AnchorPane emptyView;
    @FXML
    private Pane scriptingRootPane;
    @FXML
    private ListView<ScriptFilePropertiesDTO> scriptListView;
    private static ResourceBundle resources;
    private ExecutionViewController executionController;
    private ObservableList<ScriptFilePropertiesDTO> scriptList;

    public ScriptingViewController() {
        EventBus.register(this);
    }

    public static void showAsDialog() {
        Map<Object, Object> properties = new HashMap<>();
        properties.put(WindowProperty.WINDOW_TYPE, WindowType.PLUGIN_SETTINGS);


        if (WindowHelper.focusWindowIfAlreadyThere(properties)) {
            return;
        }

        LoaderResult<ScriptingViewController> result = load(ScriptingViewController.class, "scriptingView.fxml");
        resources = result.getResourceBundle();
        showAsDialog(result, resources.getString("scriptingViewControllerTitle"), properties, false, false,
                event -> result.getController().onCloseRequest(event),
                null, 400, 300);
    }

    private void onCloseRequest(WindowEvent event) {
        executionController.onCloseRequest();
        EventBus.unregister(this);
    }

    @FXML
    public void initialize() throws IOException {

        scriptingRootPane.getStyleClass().add(SettingsProvider.getInstance().getIconModeCssClass());
        scriptListView.setCellFactory(this::createScriptCell);

        List<ScriptFilePropertiesDTO> scriptsList = null;
        try {
            scriptsList = ScriptingProvider.getInstance().getScripts()
                    .stream()
                    .map(ScriptingTransformer::dtoToProps)
                    .toList();
        } catch (IOException e) {
            LOGGER.error("Error reading scripts. ", e);
            //TODO ioerror
        }

        scriptList = FXCollections.observableArrayList(scriptsList);
        scriptList.addListener(this::onScriptListChanged);
        scriptListView.setItems(scriptList);
        onScriptListChanged(null);

        LoaderResult<ExecutionViewController> result = ExecutionViewController.load();
        executionController = result.getController();
        executionHolder.getChildren().add(result.getMainRegion());

    }

    @SuppressWarnings("unused")
    @Subscribe(ScriptExecutionSuccessEvent.class)
    public void onScriptExecutionSuccess() {
        scriptListView.refresh();
    }


    @SuppressWarnings("unused")
    @Subscribe(ScriptExecutionProgressEvent.class)
    public void onScriptExecutionProgress() {
        scriptListView.refresh();
    }

    @SuppressWarnings("unused")
    @Subscribe(ScriptExecutionFailedEvent.class)
    public void onScriptExecutionFailed() {
        scriptListView.refresh();
    }


    private void onScriptListChanged(ListChangeListener.Change<? extends ScriptFilePropertiesDTO> changedItem) {

        // avoid endless loop on sorting
        if (changedItem != null) {
            while (changedItem.next()) {
                if (changedItem.wasPermutated()) {
                    return;
                }
            }
        }

        // keep list sorted
        FXCollections.sort(scriptListView.getItems());

        // show or hide list etc.
        if (scriptListView.getItems().isEmpty()) {
            scriptSplitPane.getItems().remove(scriptListSidebar);
            scriptSplitPane.getItems().remove(editorPane);
            if (!scriptSplitPane.getItems().contains(emptyView)) {
                scriptSplitPane.getItems().add(emptyView);
            }
        } else {
            scriptSplitPane.getItems().remove(emptyView);
            if (!scriptSplitPane.getItems().contains(scriptListSidebar)) {
                scriptSplitPane.getItems().add(scriptListSidebar);
            }
            if (!scriptSplitPane.getItems().contains(editorPane)) {
                scriptSplitPane.getItems().add(editorPane);
            }
        }

        if (scriptListView.getSelectionModel().getSelectedIndices().isEmpty()) {
            scriptListView.getSelectionModel().selectFirst();
        }
    }

    private ListCell<ScriptFilePropertiesDTO> createScriptCell(ListView<ScriptFilePropertiesDTO> scriptListView) {
        ScriptCell cell = new ScriptCell(scriptListView);
        cell.selectedProperty().addListener((observable, oldValue, newValue) -> {
            ScriptFilePropertiesDTO selectedItem = scriptListView.getSelectionModel().getSelectedItem();
            onSelectScript(selectedItem);
        });
        ScriptContextMenu contextMenu = new ScriptContextMenu(this);
        cell.setContextMenu(contextMenu);
        cell.itemProperty().addListener((observable, oldValue, newValue) -> contextMenu.setObject(newValue));
        return cell;
    }

    private void onSelectScript(ScriptFilePropertiesDTO selectedItem) {

        if (selectedItem == null)
            return;

        ScriptingViewController.ScriptEditorState editorState = editorStates.computeIfAbsent(selectedItem.getName(),
                id -> {
                    LoaderResult<SingleEditorViewController> loaderResult = SingleEditorViewController.load(this, selectedItem);
                    return new ScriptingViewController.ScriptEditorState(loaderResult.getController(), loaderResult.getMainRegion());
                });

        editorPane.getChildren().clear();
        editorPane.getChildren().add(editorState.region);
        this.executionController.filterByScript(selectedItem.getName());

        /*



        if (selectedItem == null)
            return;

        if (!selectedItem.isLoaded()) {
            ScriptFileDTO scriptFileDTO = ScriptingTransformer.propsToDTO(selectedItem);

            new ScriptLoadTask(scriptFileDTO)
                    .onSuccess(scriptCode -> onLoadScriptSucceeded(scriptFileDTO, scriptCode))
                    .onError(this::onLoadScriptFailed)
                    .run();
        } else {
            showScript(selectedItem);
        }

        ;*/

    }


    public void onNewScriptClicked() {
        showNewScriptDialog(".js");
    }

    private void showNewScriptDialog(String defaultValue) {

        String dialogTitle = resources.getString("scripting.newscript.dialog.title");

        String filename = AlertHelper.input(dialogTitle,
                resources.getString("scripting.newscript.dialog.header"),
                resources.getString("scripting.newscript.dialog.content"),
                defaultValue);

        new ScriptNewTask(filename)
                .onSuccess(this::onNewScriptCreated)
                .onError((ErrorListener<ScriptNewTask.Error>) e -> onNewScriptCreatedFailed(e, filename))
                .run();
    }

    private void onNewScriptCreated(Path path) {
        ScriptFileDTO newScriptDTO;
        try {
            newScriptDTO = ScriptingProvider.getInstance().getScripts()
                    .stream()
                    .filter(sfd -> sfd.getPath().equals(path))
                    .findFirst()
                    .orElseThrow();
        } catch (IOException e) {
            //TODO ioerror
            return;
        }

        scriptListView.getItems().add(ScriptingTransformer.dtoToProps(newScriptDTO));
    }

    private void onNewScriptCreatedFailed(ScriptNewTask.Error error, String filename) {

        String dialogTitle = resources.getString("scripting.newscript.dialog.title");

        switch (error) {
            case FILENAME_NULL -> {
                // ignore -> file chooser was cancelled
            }
            case FILENAME_EMPTY_OR_WRONG_EXTENSION -> {
                AlertHelper.warn(dialogTitle,
                        resources.getString("scripting.newscript.dialog.extension.content"),
                        true);
                showNewScriptDialog(".js");
            }
            case FILE_ALREADY_EXISTS -> {
                AlertHelper.warn(dialogTitle,
                        resources.getString("scripting.newscript.dialog.alreadyexists.content"),
                        true);

                showNewScriptDialog(filename);
            }
            case IOERROR -> {
                // TODO
            }
        }
    }

    private void renameScript(ScriptFilePropertiesDTO dto, String defaultFilename) {
        String dialogTitle = resources.getString("scripting.renamescript.dialog.title");

        String newFilename = AlertHelper.input(dialogTitle,
                resources.getString("scripting.renamescript.dialog.header"),
                resources.getString("scripting.renamescript.dialog.content"),
                defaultFilename);

        new ScriptRenameTask(ScriptingTransformer.propsToDTO(dto), newFilename)
                .onSuccess(newPath -> {
                    executionController.renameScript(dto.getName(), newFilename);
                    dto.getNameProperty().setValue(newFilename);
                    dto.getPathProperty().setValue(newPath);
                    scriptListView.refresh();
                })
                .onError((ErrorListener<ScriptRenameTask.Error>) error -> {
                    switch (error) {
                        case FILENAME_NULL -> {
                            // ignore -> file chooser was cancelled
                        }
                        case FILENAME_EMPTY_OR_WRONG_EXTENSION -> {
                            AlertHelper.warn(dialogTitle,
                                    resources.getString("scripting.renamescript.dialog.extension.content"),
                                    true);
                            renameScript(dto, newFilename);
                        }
                        case FILE_ALREADY_EXISTS -> {
                            AlertHelper.warn(dialogTitle,
                                    resources.getString("scripting.renamescript.dialog.alreadyexists.content"),
                                    true);
                            renameScript(dto, newFilename);
                        }
                        case FILENAME_NOT_CHANGED -> {
                            AlertHelper.warn(dialogTitle,
                                    resources.getString("scripting.renamescript.dialog.alreadyexists.content"), //TODO translation
                                    true);
                            renameScript(dto, newFilename);
                        }
                        case IOERROR -> {
                            // TODO
                        }
                    }
                })
                .run();
    }

    @Override
    public void renameScript(ScriptFilePropertiesDTO dto) {
        renameScript(dto, dto.getName());

    }

    @Override
    public void deleteScript(ScriptFilePropertiesDTO dto) {

        String dialogTitle = resources.getString("scripting.deletescript.dialog.title");

        if (!AlertHelper.confirm(dialogTitle,
                resources.getString("scripting.deletescript.dialog.header"),
                MessageFormat.format(resources.getString("scripting.deletescript.dialog.content"), dto.getName()),
                resources.getString("commonNoButton"),
                resources.getString("commonYesButton"))) {
            return;
        }

        new ScriptDeleteTask(ScriptingTransformer.propsToDTO(dto))
                .onSuccess(v -> scriptList.remove(dto))
                .onError((ErrorListener<ScriptDeleteTask.Error>) error -> {
                    if (error == ScriptDeleteTask.Error.IOERROR) {
                        AlertHelper.warn(dialogTitle,
                                resources.getString("scripting.deletescript.dialog.ioerror.content"), //TODO translation
                                true);
                    }
                })
                .run();
    }

    @Override
    public boolean addExecution(ScriptFilePropertiesDTO dto, ConnectionPropertiesDTO selectedConnection, String scriptCode) {
        return executionController.addExecution(dto, selectedConnection, scriptCode);
    }

    @Override
    public void runScript(ScriptFilePropertiesDTO dto) {
        //TODO
    }


}
