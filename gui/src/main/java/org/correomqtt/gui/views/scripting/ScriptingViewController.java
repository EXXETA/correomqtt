package org.correomqtt.gui.views.scripting;

import javafx.animation.Animation;
import javafx.animation.Interpolator;
import javafx.animation.RotateTransition;
import javafx.application.HostServices;
import javafx.collections.FXCollections;
import javafx.collections.ListChangeListener;
import javafx.collections.ObservableList;
import javafx.fxml.FXML;
import javafx.scene.control.ListCell;
import javafx.scene.control.ListView;
import javafx.scene.control.SplitPane;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.Pane;
import javafx.scene.layout.Region;
import javafx.scene.transform.Rotate;
import javafx.stage.WindowEvent;
import javafx.util.Duration;
import lombok.AllArgsConstructor;
import org.correomqtt.HostServicesWrapper;
import org.correomqtt.core.CoreManager;
import org.correomqtt.core.concurrent.TaskErrorResult;
import org.correomqtt.core.fileprovider.ScriptingProvider;
import org.correomqtt.core.scripting.ScriptDeleteTask;
import org.correomqtt.core.scripting.ScriptExecutionCancelledEvent;
import org.correomqtt.core.scripting.ScriptExecutionFailedEvent;
import org.correomqtt.core.scripting.ScriptExecutionProgressEvent;
import org.correomqtt.core.scripting.ScriptExecutionSuccessEvent;
import org.correomqtt.core.scripting.ScriptExecutionsDeletedEvent;
import org.correomqtt.core.scripting.ScriptFileDTO;
import org.correomqtt.core.scripting.ScriptNewTask;
import org.correomqtt.core.scripting.ScriptTaskFactories;
import org.correomqtt.core.scripting.ScriptingBackend;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import org.correomqtt.di.Observes;
import org.correomqtt.gui.controls.IconLabel;
import org.correomqtt.gui.model.ConnectionPropertiesDTO;
import org.correomqtt.gui.model.WindowProperty;
import org.correomqtt.gui.model.WindowType;
import org.correomqtt.gui.theme.ThemeManager;
import org.correomqtt.gui.utils.AlertHelper;
import org.correomqtt.gui.utils.WindowHelper;
import org.correomqtt.gui.views.LoaderResult;
import org.correomqtt.gui.views.base.BaseControllerImpl;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;
import java.nio.file.Path;
import java.text.MessageFormat;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.ResourceBundle;

import static org.correomqtt.gui.utils.JavaFxUtils.addSafeToSplitPane;

@DefaultBean
public class ScriptingViewController extends BaseControllerImpl implements ScriptContextMenuDelegate, SingleEditorViewDelegate {

    private static final Logger LOGGER = LoggerFactory.getLogger(ScriptingViewController.class);
    private static final String HELP_LINK = "https://github.com/EXXETA/correomqtt/wiki/scripting";
    private final HashMap<String, ScriptEditorState> editorStates = new HashMap<>();
    private final AlertHelper alertHelper;
    private final ScriptCellFactory scriptCellFactory;
    private final ScriptContextMenuFactory scriptContextMenuFactory;
    private final SingleEditorViewControllerFactory editorViewCtrlFactory;
    private final ScriptTaskFactories scriptTaskFactories;
    private final ExecutionViewControllerFactory executionViewControllerFactory;
    private final ScriptingProvider scriptingProvider;
    private final HostServices hostServices;
    private ResourceBundle resources;
    @FXML
    private IconLabel statusLabel;
    @FXML
    private AnchorPane executionHolder;
    @FXML
    private SplitPane mainSplitPane;
    @FXML
    private AnchorPane scriptListSidebar;
    @FXML
    private AnchorPane editorPane;
    @FXML
    private SplitPane scriptSplitPane;
    @FXML
    private AnchorPane emptyView;
    @FXML
    private Pane scriptingRootPane;
    @FXML
    private ListView<ScriptFilePropertiesDTO> scriptListView;
    private ExecutionViewController executionController;
    private ObservableList<ScriptFilePropertiesDTO> scriptList;
    private RotateTransition rotateTransition;

    @AllArgsConstructor
    private static class ScriptEditorState {
        private SingleEditorViewController controller;
        private Region region;
    }

    @Inject
    public ScriptingViewController(CoreManager coreManager,
                                   ThemeManager themeManager,
                                   AlertHelper alertHelper,
                                   ScriptCellFactory scriptCellFactory,
                                   ScriptContextMenuFactory scriptContextMenuFactory,
                                   SingleEditorViewControllerFactory editorViewCtrlFactory,
                                   ScriptTaskFactories scriptTaskFactories,
                                   ExecutionViewControllerFactory executionViewControllerFactory,
                                   ScriptingProvider scriptingProvider,
                                   HostServicesWrapper hostServicesWrapper
    ) {
        super(coreManager, themeManager);
        this.alertHelper = alertHelper;
        this.scriptCellFactory = scriptCellFactory;
        this.scriptContextMenuFactory = scriptContextMenuFactory;
        this.editorViewCtrlFactory = editorViewCtrlFactory;
        this.scriptTaskFactories = scriptTaskFactories;
        this.executionViewControllerFactory = executionViewControllerFactory;
        this.scriptingProvider = scriptingProvider;
        this.hostServices = hostServicesWrapper.getHostServices();
    }

    public void showAsDialog() {
        Map<Object, Object> properties = new HashMap<>();
        properties.put(WindowProperty.WINDOW_TYPE, WindowType.PLUGIN_SETTINGS);
        if (WindowHelper.focusWindowIfAlreadyThere(properties)) {
            return;
        }
        LoaderResult<ScriptingViewController> result = load(ScriptingViewController.class, "scriptingView.fxml", () -> this);
        resources = result.getResourceBundle();
        showAsDialog(result, resources.getString("scriptingViewControllerTitle"), properties, false, false,
                event -> result.getController().onCloseRequest(event),
                null, 400, 300);
    }

    private void onCloseRequest(WindowEvent event) {
        if (scriptListView.getItems().stream()
                .anyMatch(ScriptFilePropertiesDTO::isDirty) &&
                !alertHelper.confirm(
                        resources.getString("scriptingUnsavedCheckTitle"),
                        null,
                        resources.getString("scriptingUnsavedCheckDescription"),
                        resources.getString("commonNoButton"),
                        resources.getString("commonYesButton")
                )) {
            event.consume();
            return;
        }
        executionController.cleanup();
    }

    @FXML
    private void initialize() throws IOException {
        rotateTransition = new RotateTransition();
        rotateTransition.setAxis(Rotate.Z_AXIS);
        rotateTransition.setByAngle(360);
        rotateTransition.setCycleCount(Animation.INDEFINITE);
        rotateTransition.setDuration(Duration.millis(1000));
        rotateTransition.setNode(statusLabel.getGraphic());
        rotateTransition.setInterpolator(Interpolator.LINEAR);
        scriptingRootPane.getStyleClass().add(themeManager.getIconModeCssClass());
        scriptListView.setCellFactory(this::createScriptCell);
        List<ScriptFilePropertiesDTO> scriptsList = null;
        try {
            scriptsList = scriptingProvider.getScripts()
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
        LoaderResult<ExecutionViewController> result = executionViewControllerFactory.create().load();
        executionController = result.getController();
        executionHolder.getChildren().add(result.getMainRegion());
        updateStatusLabel();
    }

    private ListCell<ScriptFilePropertiesDTO> createScriptCell(ListView<ScriptFilePropertiesDTO> scriptListView) {
        ScriptCell cell = scriptCellFactory.create(scriptListView);
        cell.selectedProperty().addListener((observable, oldValue, newValue) -> {
            ScriptFilePropertiesDTO selectedItem = scriptListView.getSelectionModel().getSelectedItem();
            onSelectScript(selectedItem);
        });
        ScriptContextMenu contextMenu = scriptContextMenuFactory.create(this);
        cell.setContextMenu(contextMenu);
        cell.itemProperty().addListener((observable, oldValue, newValue) -> contextMenu.setObject(newValue));
        return cell;
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
            mainSplitPane.getItems().remove(executionHolder);
            addSafeToSplitPane(scriptSplitPane, emptyView);
        } else {
            scriptSplitPane.getItems().remove(emptyView);
            addSafeToSplitPane(scriptSplitPane, scriptListSidebar);
            addSafeToSplitPane(scriptSplitPane, editorPane);
            addSafeToSplitPane(mainSplitPane, executionHolder);
        }
        if (scriptListView.getSelectionModel().getSelectedIndices().isEmpty()) {
            scriptListView.getSelectionModel().selectFirst();
        }
    }

    private void updateStatusLabel() {
        List<ExecutionPropertiesDTO> executions = ScriptingBackend.getExecutions()
                .stream()
                .map(ExecutionTransformer::dtoToProps)
                .toList();
        long running = executions.stream()
                .filter(e -> e.getState() == ScriptState.RUNNING)
                .count();
        String description;
        if (running == 0) {
            rotateTransition.stop();
            statusLabel.setIcon("mdi-script");
            statusLabel.getGraphic().setRotate(0);
            rotateTransition.setNode(statusLabel.getGraphic());
            description = MessageFormat.format("{0} finished", executions.size());
        } else {
            statusLabel.setIcon("mdi-loading");
            rotateTransition.setNode(statusLabel.getGraphic());
            rotateTransition.play();
            description = MessageFormat.format("{0} running / {1} finished", running, executions.size() - running);
        }
        statusLabel.setText(description);
    }

    private void onSelectScript(ScriptFilePropertiesDTO selectedItem) {
        if (selectedItem == null)
            return;
        ScriptingViewController.ScriptEditorState editorState = editorStates.computeIfAbsent(selectedItem.getName(),
                id -> {
                    LoaderResult<SingleEditorViewController> loaderResult = editorViewCtrlFactory.create(this, selectedItem).load();
                    return new ScriptingViewController.ScriptEditorState(loaderResult.getController(), loaderResult.getMainRegion());
                });
        editorPane.getChildren().clear();
        editorPane.getChildren().add(editorState.region);
        executionController.filterByScript(selectedItem.getName());
    }

    @SuppressWarnings("unused")
    @Observes({
            ScriptExecutionCancelledEvent.class,
            ScriptExecutionSuccessEvent.class,
            ScriptExecutionProgressEvent.class,
            ScriptExecutionFailedEvent.class,
            ScriptExecutionsDeletedEvent.class
    })
    public void onScriptExecutionFinished() {
        scriptListView.refresh();
        updateStatusLabel();
    }

    public void onNewScriptClicked() {
        showNewScriptDialog(".js");
    }

    private void showNewScriptDialog(String defaultValue) {
        String dialogTitle = resources.getString("scripting.newscript.dialog.title");
        String filename = alertHelper.input(dialogTitle,
                resources.getString("scripting.newscript.dialog.header"),
                resources.getString("scripting.newscript.dialog.content"),
                defaultValue);
        scriptTaskFactories.getNewFactory().create(filename)
                .onSuccess(this::onNewScriptCreated)
                .onError(r -> onNewScriptCreatedFailed(r, filename))
                .run();
    }

    private void onNewScriptCreated(Path path) {
        ScriptFileDTO newScriptDTO;
        try {
            newScriptDTO = scriptingProvider.getScripts()
                    .stream()
                    .filter(sfd -> sfd.getPath().equals(path))
                    .findFirst()
                    .orElseThrow();
        } catch (IOException e) {
            //TODO ioerror
            return;
        }
        ScriptFilePropertiesDTO dto = ScriptingTransformer.dtoToProps(newScriptDTO);
        scriptListView.getItems().add(dto);
        scriptListView.getSelectionModel().select(dto);
        onSelectScript(dto);
    }

    private void onNewScriptCreatedFailed(TaskErrorResult<ScriptNewTask.Error> result, String filename) {
        String dialogTitle = resources.getString("scripting.newscript.dialog.title");
        if (result.isExpected()) {
            switch (result.getExpectedError()) {
                case FILENAME_NULL -> {
                    // ignore -> file chooser was cancelled
                }
                case FILENAME_EMPTY_OR_WRONG_EXTENSION -> {
                    alertHelper.warn(dialogTitle,
                            resources.getString("scripting.newscript.dialog.extension.content"),
                            true);
                    showNewScriptDialog(".js");
                }
                case FILE_ALREADY_EXISTS -> {
                    alertHelper.warn(dialogTitle,
                            resources.getString("scripting.newscript.dialog.alreadyexists.content"),
                            true);
                    showNewScriptDialog(filename);
                }
                case IOERROR -> {
                    // TODO
                }
            }
        } else {
            alertHelper.unexpectedAlert(result.getUnexpectedError());
        }
    }

    private void renameScript(ScriptFilePropertiesDTO dto, String defaultFilename) {
        String dialogTitle = resources.getString("scripting.renamescript.dialog.title");
        String newFilename = alertHelper.input(dialogTitle,
                resources.getString("scripting.renamescript.dialog.header"),
                resources.getString("scripting.renamescript.dialog.content"),
                defaultFilename);
        scriptTaskFactories.getRenameFactory().create(ScriptingTransformer.propsToDTO(dto), newFilename)
                .onSuccess(newPath -> {
                    executionController.renameScript(dto.getName(), newFilename);
                    dto.getNameProperty().setValue(newFilename);
                    dto.getPathProperty().setValue(newPath);
                    scriptListView.refresh();
                    updateStatusLabel();
                })
                .onError(r -> {
                    if (r.isExpected()) {
                        switch (r.getExpectedError()) {
                            case FILENAME_NULL -> {
                                // ignore -> file chooser was cancelled
                            }
                            case FILENAME_EMPTY_OR_WRONG_EXTENSION -> {
                                alertHelper.warn(dialogTitle,
                                        resources.getString("scripting.renamescript.dialog.extension.content"),
                                        true);
                                renameScript(dto, newFilename);
                            }
                            case FILE_ALREADY_EXISTS -> {
                                alertHelper.warn(dialogTitle,
                                        resources.getString("scripting.renamescript.dialog.alreadyexists.content"),
                                        true);
                                renameScript(dto, newFilename);
                            }
                            case FILENAME_NOT_CHANGED -> {
                                alertHelper.warn(dialogTitle,
                                        resources.getString("scripting.renamescript.dialog.alreadyexists.content"), //TODO translation
                                        true);
                                renameScript(dto, newFilename);
                            }
                            case IOERROR -> {
                                // TODO
                            }
                        }
                    } else {
                        alertHelper.unexpectedAlert(r.getUnexpectedError());
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
        if (!alertHelper.confirm(dialogTitle,
                resources.getString("scripting.deletescript.dialog.header"),
                MessageFormat.format(resources.getString("scripting.deletescript.dialog.content"), dto.getName()),
                resources.getString("commonNoButton"),
                resources.getString("commonYesButton"))) {
            return;
        }
        scriptTaskFactories.getDeleteFactory().create(ScriptingTransformer.propsToDTO(dto))
                .onSuccess(() -> onScriptDeleted(dto))
                .onError(r -> {
                    if (r.isExpected()) {
                        if (r.getExpectedError() == ScriptDeleteTask.Error.IOERROR) {
                            alertHelper.warn(dialogTitle,
                                    resources.getString("scripting.deletescript.dialog.ioerror.content"), //TODO translation
                                    true);
                        }
                    } else {
                        alertHelper.unexpectedAlert(r.getUnexpectedError());
                    }
                })
                .run();
    }

    private void onScriptDeleted(ScriptFilePropertiesDTO dto) {
        scriptListView.getItems().remove(dto);
        scriptListView.refresh();
        updateStatusLabel();
        if (scriptListView.getSelectionModel().getSelectedIndices().isEmpty()) {
            scriptListView.getSelectionModel().selectFirst();
        }
    }

    @Override
    public void runScript(ScriptFilePropertiesDTO dto) {
        //TODO
    }

    @Override
    public boolean addExecution(ScriptFilePropertiesDTO dto, ConnectionPropertiesDTO selectedConnection, String scriptCode) {
        return executionController.addExecution(dto, selectedConnection, scriptCode);
    }

    @Override
    public void onPlainTextChange(ScriptFilePropertiesDTO dto) {
        scriptListView.refresh();
        updateStatusLabel();
    }

    @FXML
    private void onHelpLinkClicked() {
        hostServices.showDocument(HELP_LINK);
    }
}
