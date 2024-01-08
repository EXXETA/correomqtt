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
import javafx.stage.WindowEvent;
import org.correomqtt.business.concurrent.ErrorListener;
import org.correomqtt.business.connection.ConnectionState;
import org.correomqtt.business.connection.ConnectionStateChangedEvent;
import org.correomqtt.business.eventbus.EventBus;
import org.correomqtt.business.eventbus.Subscribe;
import org.correomqtt.business.eventbus.SubscribeFilter;
import org.correomqtt.business.fileprovider.ScriptingProvider;
import org.correomqtt.business.fileprovider.SettingsProvider;
import org.correomqtt.business.mqtt.CorreoMqttClient;
import org.correomqtt.business.scripting.ExecutionDTO;
import org.correomqtt.business.scripting.ScriptDeleteTask;
import org.correomqtt.business.scripting.ScriptExecutionFailedEvent;
import org.correomqtt.business.scripting.ScriptExecutionProgressEvent;
import org.correomqtt.business.scripting.ScriptExecutionSuccessEvent;
import org.correomqtt.business.scripting.ScriptFileDTO;
import org.correomqtt.business.scripting.ScriptLoadTask;
import org.correomqtt.business.scripting.ScriptNewTask;
import org.correomqtt.business.scripting.ScriptRenameTask;
import org.correomqtt.business.scripting.ScriptSaveTask;
import org.correomqtt.business.utils.ConnectionHolder;
import org.correomqtt.gui.controls.IconButton;
import org.correomqtt.gui.model.ConnectionPropertiesDTO;
import org.correomqtt.gui.transformer.ConnectionTransformer;
import org.correomqtt.gui.utils.AlertHelper;
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
import java.util.List;
import java.util.ResourceBundle;

import static org.correomqtt.business.eventbus.SubscribeFilterNames.SCRIPT_NAME;

public class SingleEditorViewController extends BaseControllerImpl {

    private static final Logger LOGGER = LoggerFactory.getLogger(SingleEditorViewController.class);
    private final SingleEditorViewDelegate delegate;
    private final ScriptFilePropertiesDTO scriptFilePropertiesDTO;
    @FXML
    public AnchorPane scriptEditor;
    @FXML
    public AnchorPane emptyView;
    @FXML
    public IconButton scriptingSaveButton;
    @FXML
    private CodeArea codeArea;
    @FXML
    private Pane scriptingViewCodeAreaPane;
    @FXML
    private Button scriptingRunButton;
    @FXML
    private ComboBox<ConnectionPropertiesDTO> connectionList;
    private static ResourceBundle resources;

    public SingleEditorViewController(SingleEditorViewDelegate delegate, ScriptFilePropertiesDTO scriptFilePropertiesDTO) {
        this.delegate = delegate;
        this.scriptFilePropertiesDTO = scriptFilePropertiesDTO;
        EventBus.register(this); //TODO cleanup
    }

    public static LoaderResult<SingleEditorViewController> load(SingleEditorViewDelegate delegate, ScriptFilePropertiesDTO scriptFilePropertiesDTO) {

        LoaderResult<SingleEditorViewController> result = load(SingleEditorViewController.class, "singleEditorView.fxml",
                () -> new SingleEditorViewController(delegate, scriptFilePropertiesDTO));
        resources = result.getResourceBundle();
        return result;
    }

    @FXML
    public void initialize() throws IOException {

        updateConnections();

        ScriptFileDTO scriptFileDTO = ScriptingTransformer.propsToDTO(scriptFilePropertiesDTO);
        new ScriptLoadTask(scriptFileDTO)
                .onSuccess(scriptCode -> onLoadScriptSucceeded(scriptFileDTO, scriptCode))
                .onError(this::onLoadScriptFailed)
                .run();

        connectionList.setCellFactory(ConnectionCell::new);
        connectionList.setButtonCell(new ConnectionCellButton(null));


    }

    private void onLoadScriptFailed(ScriptLoadTask.Error error) {
        // TODO ioerror
    }

    private void onLoadScriptSucceeded(ScriptFileDTO scriptFileDTO, String scriptCode) {
        scriptFilePropertiesDTO.getCodeProperty().set(scriptCode);
        codeArea.replaceText(scriptCode);
        scriptingViewCodeAreaPane.setManaged(true);
        scriptingViewCodeAreaPane.setVisible(true);
        scriptingViewCodeAreaPane.getChildren().add(new VirtualizedScrollPane<>(codeArea));
        codeArea.prefWidthProperty().bind(scriptingViewCodeAreaPane.widthProperty());
        codeArea.prefHeightProperty().bind(scriptingViewCodeAreaPane.heightProperty());
        scriptingRunButton.setDisable(false);
        codeArea.setDisable(false);
        codeArea.plainTextChanges()
                .filter(ch -> !ch.isIdentity())
                .successionEnds(Duration.ofMillis(500))
                .subscribe(ignore -> {
                    if (!scriptFilePropertiesDTO.isDirty()) {
                        LOGGER.info("Set dirty on {}", scriptFilePropertiesDTO.getName());
                        scriptFilePropertiesDTO.getDirtyProperty().set(true);
                        scriptingSaveButton.setDisable(false);
                    }
                });
    }

    public void onRunClicked() {
        runScript(scriptFilePropertiesDTO);
    }


    public void runScript(ScriptFilePropertiesDTO dto) {

        ConnectionPropertiesDTO selectedConnection = connectionList.getSelectionModel().getSelectedItem();

        if (delegate.addExecution(dto, selectedConnection, codeArea.getText())) {
            scriptingRunButton.setDisable(true);
        }

    }


    @Subscribe(ConnectionStateChangedEvent.class)
    public void onConnectionChangedEvent() {
        updateConnections();

    }

    private void updateConnections() {

        ConnectionPropertiesDTO selectedItem = connectionList.getSelectionModel().getSelectedItem();

        connectionList.setItems(FXCollections.observableArrayList(
                ConnectionTransformer.dtoListToPropList(ConnectionHolder.getInstance().getSortedConnections())
        ));

        if (selectedItem == null) {
            selectedItem = connectionList.getItems().stream()
                    .filter(c -> {
                        CorreoMqttClient client = ConnectionHolder.getInstance().getClient(c.getId());
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

    public void onSaveClicked(ActionEvent actionEvent) {

        new ScriptSaveTask(ScriptingTransformer.propsToDTO(scriptFilePropertiesDTO), codeArea.getText())
                .onSuccess(v -> onSaveSuccess())
                .onError(this::onSaveFailed)
                .run();

    }

    private void onSaveFailed(ScriptSaveTask.Error error) {
        //TODO
    }

    private void onSaveSuccess() {
        scriptingSaveButton.setDisable(true);
        scriptFilePropertiesDTO.getDirtyProperty().set(false);
    }

    @SuppressWarnings("unused")
    @Subscribe(ScriptExecutionSuccessEvent.class)
    public void onScriptExecutionSuccess() {
        scriptingRunButton.setDisable(false);
    }

    @SuppressWarnings("unused")
    @Subscribe(ScriptExecutionFailedEvent.class)
    public void onScriptExecutionFailed() {

        scriptingRunButton.setDisable(false);
    }


    @SubscribeFilter(SCRIPT_NAME)
    public String getFileName(){
        return scriptFilePropertiesDTO.getName();
    }

    public void onRenameClicked(ActionEvent actionEvent) {
        delegate.renameScript(scriptFilePropertiesDTO);
    }

    public void onDeleteClicked(ActionEvent actionEvent) {
        delegate.deleteScript(scriptFilePropertiesDTO);
    }
}
