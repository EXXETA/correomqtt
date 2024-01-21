package org.correomqtt.gui.views.scripting;

import dagger.assisted.Assisted;
import dagger.assisted.AssistedFactory;
import dagger.assisted.AssistedInject;
import javafx.collections.FXCollections;
import javafx.fxml.FXML;
import javafx.scene.control.Button;
import javafx.scene.control.ComboBox;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.Pane;
import org.correomqtt.core.settings.SettingsProvider;
import org.correomqtt.core.concurrent.TaskErrorResult;
import org.correomqtt.core.connection.ConnectionState;
import org.correomqtt.core.connection.ConnectionStateChangedEvent;
import org.correomqtt.core.eventbus.EventBus;
import org.correomqtt.core.eventbus.Subscribe;
import org.correomqtt.core.eventbus.SubscribeFilter;
import org.correomqtt.core.mqtt.CorreoMqttClient;
import org.correomqtt.core.scripting.ScriptExecutionCancelledEvent;
import org.correomqtt.core.scripting.ScriptExecutionFailedEvent;
import org.correomqtt.core.scripting.ScriptExecutionSuccessEvent;
import org.correomqtt.core.scripting.ScriptFileDTO;
import org.correomqtt.core.scripting.ScriptLoadTask;
import org.correomqtt.core.scripting.ScriptSaveTask;
import org.correomqtt.core.scripting.ScriptingBackend;
import org.correomqtt.core.utils.ConnectionHolder;
import org.correomqtt.gui.controls.IconButton;
import org.correomqtt.gui.model.ConnectionPropertiesDTO;
import org.correomqtt.gui.theme.ThemeManager;
import org.correomqtt.gui.transformer.ConnectionTransformer;
import org.correomqtt.gui.utils.AlertHelper;
import org.correomqtt.gui.views.LoaderResult;
import org.correomqtt.gui.views.base.BaseControllerImpl;
import org.fxmisc.flowless.VirtualizedScrollPane;
import org.fxmisc.richtext.CodeArea;
import org.fxmisc.richtext.model.PlainTextChange;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.time.Duration;
import java.util.List;
import java.util.ResourceBundle;
import java.util.concurrent.atomic.AtomicBoolean;

import static org.correomqtt.core.eventbus.SubscribeFilterNames.SCRIPT_NAME;

public class SingleEditorViewController extends BaseControllerImpl {

    private static final Logger LOGGER = LoggerFactory.getLogger(SingleEditorViewController.class);
    private final ConnectionHolder connectionHolder;
    private final ConnectionCellButton.Factory connectionCellButtonFactory;
    private final AlertHelper alertHelper;
    private final ScriptLoadTask.Factory scriptLoadTaskFactory;
    private final ScriptSaveTask.Factory scriptSaveTaskFactory;
    private final SingleEditorViewDelegate delegate;
    private final ScriptFilePropertiesDTO scriptFilePropertiesDTO;
    private final AtomicBoolean revert = new AtomicBoolean(false);
    @FXML
    private AnchorPane scriptEditor;
    @FXML
    private AnchorPane emptyView;
    @FXML
    private IconButton scriptingSaveButton;
    @FXML
    private IconButton scriptingRevertButton;
    @FXML
    private IconButton scriptingRenameButton;
    @FXML
    private IconButton scriptingDeleteButton;
    @FXML
    private CodeArea codeArea;
    @FXML
    private Pane scriptingViewCodeAreaPane;
    @FXML
    private Button scriptingRunButton;
    @FXML
    private ComboBox<ConnectionPropertiesDTO> connectionList;
    private ResourceBundle resources;

    @AssistedFactory
    public interface Factory {
        SingleEditorViewController create(SingleEditorViewDelegate delegate,
                                          ScriptFilePropertiesDTO scriptFilePropertiesDTO);

    }

    @AssistedInject
    public SingleEditorViewController(ConnectionHolder connectionHolder,
                                      SettingsProvider settingsProvider,
                                      ThemeManager themeManager,
                                      ConnectionCellButton.Factory connectionCellButtonFactory,
                                      AlertHelper alertHelper,
                                      ScriptLoadTask.Factory scriptLoadTaskFactory,
                                      ScriptSaveTask.Factory scriptSaveTaskFactory,
                                      @Assisted SingleEditorViewDelegate delegate,
                                      @Assisted ScriptFilePropertiesDTO scriptFilePropertiesDTO) {
        super(settingsProvider, themeManager);
        this.connectionHolder = connectionHolder;
        this.connectionCellButtonFactory = connectionCellButtonFactory;
        this.alertHelper = alertHelper;
        this.scriptLoadTaskFactory = scriptLoadTaskFactory;
        this.scriptSaveTaskFactory = scriptSaveTaskFactory;
        this.delegate = delegate;
        this.scriptFilePropertiesDTO = scriptFilePropertiesDTO;
        EventBus.register(this);
    }

    public LoaderResult<SingleEditorViewController> load() {

        LoaderResult<SingleEditorViewController> result = load(SingleEditorViewController.class, "singleEditorView.fxml", () -> this);
        resources = result.getResourceBundle();
        return result;
    }

    @FXML
    private void initialize() {

        updateConnections();

        ScriptFileDTO scriptFileDTO = ScriptingTransformer.propsToDTO(scriptFilePropertiesDTO);
        scriptLoadTaskFactory.create(scriptFileDTO)
                .onSuccess(scriptCode -> onLoadScriptSucceeded(scriptFileDTO, scriptCode))
                .onError(this::onLoadScriptFailed)
                .run();

        connectionList.setCellFactory(connectionCellButtonFactory::create);
        connectionList.setButtonCell(connectionCellButtonFactory.create(null));

    }

    private void updateConnections() {

        ConnectionPropertiesDTO selectedItem = connectionList.getSelectionModel().getSelectedItem();

        connectionList.setItems(FXCollections.observableArrayList(
                ConnectionTransformer.dtoListToPropList(connectionHolder.getSortedConnections())
        ));

        if (selectedItem == null) {
            selectedItem = connectionList.getItems().stream()
                    .filter(c -> {
                        CorreoMqttClient client = connectionHolder.getClient(c.getId());
                        return client != null && client.getState() == ConnectionState.CONNECTED;
                    })
                    .findFirst()
                    .orElseGet(() -> {
                        if (!connectionList.getItems().isEmpty()) {
                            return connectionList.getItems().get(0);
                        }
                        return null;
                    });
        }

        if (selectedItem != null) {

            String selectedConnectionId = selectedItem.getId();
            ConnectionPropertiesDTO newItemToSelect = connectionList.getItems().stream()
                    .filter(c -> c.getId().equals(selectedConnectionId))
                    .findFirst()
                    .orElseThrow();


            connectionList.getSelectionModel().select(newItemToSelect);
        }

    }

    private void onLoadScriptSucceeded(ScriptFileDTO scriptFileDTO, String scriptCode) {
        scriptFilePropertiesDTO.getCodeProperty().set(scriptCode);
        codeArea.replaceText(scriptCode);
        scriptingViewCodeAreaPane.setManaged(true);
        scriptingViewCodeAreaPane.setVisible(true);
        scriptingViewCodeAreaPane.getChildren().add(new VirtualizedScrollPane<>(codeArea));
        codeArea.prefWidthProperty().bind(scriptingViewCodeAreaPane.widthProperty());
        codeArea.prefHeightProperty().bind(scriptingViewCodeAreaPane.heightProperty());

        List<ExecutionPropertiesDTO> executions = ScriptingBackend.getExecutions()
                .stream()
                .filter(e -> scriptFileDTO.getName().equals(e.getScriptFile().getName()))
                .map(ExecutionTransformer::dtoToProps)
                .toList();

        long running = executions.stream()
                .filter(e -> e.getState() == ScriptState.RUNNING)
                .count();
        disableActionsOnRunningScript(running > 0);
        codeArea.setDisable(false);
        codeArea.plainTextChanges()
                .filter(ch -> !ch.isIdentity())
                .successionEnds(Duration.ofMillis(500))
                .subscribe(this::onPlainTextChanges);
    }

    private void onLoadScriptFailed(TaskErrorResult<ScriptLoadTask.Error> result) {
        alertHelper.unexpectedAlert(result.getUnexpectedError());
    }

    public void disableActionsOnRunningScript(boolean disable) {
        scriptingRunButton.setDisable(disable);
        scriptingRenameButton.setDisable(disable);
        scriptingDeleteButton.setDisable(disable);
        scriptingSaveButton.setDisable(disable);
    }

    private void onPlainTextChanges(PlainTextChange change) {
        if (!scriptFilePropertiesDTO.isDirty() && !revert.get()) {
            LOGGER.info("Set dirty on {}", scriptFilePropertiesDTO.getName());
            scriptFilePropertiesDTO.getDirtyProperty().set(true);
            scriptingSaveButton.setDisable(false);
            scriptingRevertButton.setDisable(false);
            delegate.onPlainTextChange(scriptFilePropertiesDTO);
        }
        if (revert.get()) {
            revert.set(false);
        }
    }

    public void onRunClicked() {
        runScript(scriptFilePropertiesDTO);
    }

    public void runScript(ScriptFilePropertiesDTO dto) {

        ConnectionPropertiesDTO selectedConnection = connectionList.getSelectionModel().getSelectedItem();

        if (delegate.addExecution(dto, selectedConnection, codeArea.getText())) {
            disableActionsOnRunningScript(true);
        }

    }

    @Subscribe(ConnectionStateChangedEvent.class)
    public void onConnectionChangedEvent() {
        updateConnections();

    }

    public void onSaveClicked() {
        scriptSaveTaskFactory.create(ScriptingTransformer.propsToDTO(scriptFilePropertiesDTO), codeArea.getText())
                .onSuccess(() -> onSaveSuccess(codeArea.getText()))
                .onError(this::onSaveFailed)
                .run();
    }

    private void onSaveSuccess(String code) {
        scriptingSaveButton.setDisable(true);
        scriptingRevertButton.setDisable(true);
        scriptFilePropertiesDTO.getDirtyProperty().set(false);
        scriptFilePropertiesDTO.getCodeProperty().set(code);
        delegate.onPlainTextChange(scriptFilePropertiesDTO);
    }

    private void onSaveFailed(TaskErrorResult<ScriptSaveTask.Error> error) {
        alertHelper.unexpectedAlert(error.getUnexpectedError());
    }

    @SuppressWarnings("unused")
    @Subscribe(ScriptExecutionCancelledEvent.class)
    public void onScriptExecutionCancelled() {
        disableActionsOnRunningScript(false);
    }


    @SuppressWarnings("unused")
    @Subscribe(ScriptExecutionSuccessEvent.class)
    public void onScriptExecutionSuccess() {
        disableActionsOnRunningScript(false);
    }

    @SuppressWarnings("unused")
    @Subscribe(ScriptExecutionFailedEvent.class)
    public void onScriptExecutionFailed() {
        disableActionsOnRunningScript(false);
    }


    @SubscribeFilter(SCRIPT_NAME)
    public String getFileName() {
        return scriptFilePropertiesDTO.getName();
    }

    public void onRenameClicked() {
        delegate.renameScript(scriptFilePropertiesDTO);
    }

    public void onDeleteClicked() {
        delegate.deleteScript(scriptFilePropertiesDTO);
    }

    public void onRevertClicked() {
        revert.set(true);
        codeArea.replaceText(scriptFilePropertiesDTO.getCode());
        scriptFilePropertiesDTO.getDirtyProperty().set(false);
        scriptingSaveButton.setDisable(true);
        scriptingRevertButton.setDisable(true);
        delegate.onPlainTextChange(scriptFilePropertiesDTO);
    }

    public void cleanUp() {
        EventBus.unregister(this);
    }
}
