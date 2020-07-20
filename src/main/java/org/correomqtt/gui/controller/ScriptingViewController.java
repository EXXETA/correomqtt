package org.correomqtt.gui.controller;

import javafx.collections.FXCollections;
import javafx.collections.ObservableList;
import javafx.event.ActionEvent;
import javafx.fxml.FXML;
import javafx.scene.control.*;
import javafx.scene.layout.Pane;
import org.correomqtt.business.dispatcher.*;
import org.correomqtt.business.exception.CorreoMqttScriptExecutionFailed;
import org.correomqtt.business.model.ScriptExecutionDTO;
import org.correomqtt.business.model.ScriptingDTO;
import org.correomqtt.business.mqtt.CorreoMqttClient;
import org.correomqtt.business.mqtt.CorreoMqttClientState;
import org.correomqtt.business.provider.ScriptingProvider;
import org.correomqtt.business.provider.SettingsProvider;
import org.correomqtt.business.utils.ConnectionHolder;
import org.correomqtt.gui.business.TaskFactory;
import org.correomqtt.gui.cell.ConnectionCell;
import org.correomqtt.gui.cell.ConnectionCellButton;
import org.correomqtt.gui.cell.ScriptCell;
import org.correomqtt.gui.helper.AlertHelper;
import org.correomqtt.gui.model.ConnectionPropertiesDTO;
import org.correomqtt.gui.model.ScriptingPropertiesDTO;
import org.correomqtt.gui.model.WindowProperty;
import org.correomqtt.gui.model.WindowType;
import org.correomqtt.gui.transformer.ConnectionTransformer;
import org.correomqtt.gui.transformer.ScriptingTransformer;
import org.correomqtt.gui.utils.CodeAreaOutputStream;
import org.correomqtt.gui.utils.WindowHelper;
import org.fxmisc.flowless.VirtualizedScrollPane;
import org.fxmisc.richtext.CodeArea;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;
import java.text.MessageFormat;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.ResourceBundle;
import java.util.stream.Collectors;

public class ScriptingViewController extends BaseController implements ScriptLoadObserver, ScriptSubmitObserver, ScriptCancelObserver, ScriptResultObserver {

    private static final Logger LOGGER = LoggerFactory.getLogger(ScriptingViewController.class);

    @FXML
    private Pane scriptingRootPane;

    @FXML
    private CodeArea codeArea;

    @FXML
    private CodeArea logArea;

    @FXML
    private Pane scriptingViewCodeAreaPane;

    @FXML
    private Pane scriptingViewLogAreaPane;

    @FXML
    private Label statusText;

    @FXML
    private ListView<ScriptingPropertiesDTO> scriptListView;

    @FXML
    private Button scriptingRunButton;

    @FXML
    private Button scriptingStopButton;

    @FXML
    private ComboBox<ConnectionPropertiesDTO> connectionList;

    private static ResourceBundle resources;

    private ObservableList<ScriptingPropertiesDTO> scripts;

    public ScriptingViewController() {
        ScriptLoadDispatcher.getInstance().addObserver(this);
        ScriptSubmitDispatcher.getInstance().addObserver(this);
        ScriptCancelDispatcher.getInstance().addObserver(this);
        ScriptResultDispatcher.getInstance().addObserver(this);
    }

    public static void showAsDialog() {
        Map<Object, Object> properties = new HashMap<>();
        properties.put(WindowProperty.WINDOW_TYPE, WindowType.PLUGIN_SETTINGS);


        if (WindowHelper.focusWindowIfAlreadyThere(properties)) {
            return;
        }

        LoaderResult<ScriptingViewController> result = load(ScriptingViewController.class, "scriptingView.fxml");
        resources = result.getResourceBundle();
        showAsDialog(result, resources.getString("scriptingViewControllerTitle"), properties, 400, 300, false, null, null);
    }

    @FXML
    public void initialize() throws IOException {

        scriptingRootPane.getStyleClass().add(SettingsProvider.getInstance().getIconModeCssClass());

        scriptingViewCodeAreaPane.setVisible(false);
        scriptingViewLogAreaPane.setVisible(false);

        scriptListView.setCellFactory(this::createCell);

        List<ScriptingPropertiesDTO> scriptsList = ScriptingProvider.getInstance().getScripts()
                .stream()
                .map(ScriptingTransformer::dtoToProps)
                .collect(Collectors.toList());

        scripts = FXCollections.observableList(scriptsList);
        scriptListView.setItems(scripts);


        connectionList.setCellFactory(ConnectionCell::new);
        connectionList.setButtonCell(new ConnectionCellButton(null));

        ObservableList<ConnectionPropertiesDTO> connectionItems = FXCollections.observableArrayList(
                ConnectionTransformer.dtoListToPropList(ConnectionHolder.getInstance().getSortedConnections())
        );

        connectionList.setItems(connectionItems);

        ConnectionPropertiesDTO selectedItem = connectionItems.stream()
                .filter(c -> {
                    CorreoMqttClient client = ConnectionHolder.getInstance().getClient(c.getId());
                    if (client == null) {
                        return false;
                    }
                    return client.getState() == CorreoMqttClientState.CONNECTED;
                })
                .findFirst()
                .orElseGet(() -> {
                    if (connectionItems.size() > 0) {
                        return connectionItems.get(0);
                    }
                    return null;
                });

        connectionList.getSelectionModel().select(selectedItem);

    }


    private ListCell<ScriptingPropertiesDTO> createCell(ListView<ScriptingPropertiesDTO> scriptListView) {
        ScriptCell cell = new ScriptCell(scriptListView);
        cell.selectedProperty().addListener((observable, oldValue, newValue) -> {
            ScriptingPropertiesDTO selectedItem = scriptListView.getSelectionModel().getSelectedItem();
            TaskFactory.loadScript(selectedItem);
        });
        return cell;
    }

    @Override
    public void onLoadScriptSucceeded(ScriptingDTO scriptingDTO, String scriptCode) {
        scripts.stream()
                .filter(script -> script.getPath().equals(scriptingDTO.getPath()))
                .findFirst()
                .ifPresent(script -> {
                    script.getCodeProperty().set(scriptCode);
                    showScript(script);
                });
    }

    private void showScript(ScriptingPropertiesDTO script) {
        codeArea.replaceText(script.getCode());
        scriptingViewCodeAreaPane.setManaged(true);
        scriptingViewCodeAreaPane.setVisible(true);
        scriptingViewCodeAreaPane.getChildren().add(new VirtualizedScrollPane<>(codeArea));
        codeArea.prefWidthProperty().bind(scriptingViewCodeAreaPane.widthProperty());
        codeArea.prefHeightProperty().bind(scriptingViewCodeAreaPane.heightProperty());

        logArea.replaceText("");
        scriptingViewLogAreaPane.setManaged(true);
        scriptingViewLogAreaPane.setVisible(true);
        scriptingViewLogAreaPane.getChildren().add(new VirtualizedScrollPane<>(logArea));
        logArea.prefWidthProperty().bind(scriptingViewLogAreaPane.widthProperty());
        logArea.prefHeightProperty().bind(scriptingViewLogAreaPane.heightProperty());

        scriptingRunButton.setDisable(false);
    }

    @Override
    public void onLoadScriptCancelled(ScriptingDTO scriptingDTO) {

    }

    @Override
    public void onLoadScriptFailed(ScriptingDTO scriptingDTO, Throwable exception) {

    }

    public void onRunClicked(ActionEvent actionEvent) {
        CodeAreaOutputStream out = new CodeAreaOutputStream(logArea);
        ConnectionPropertiesDTO selectedConnection = connectionList.getSelectionModel().getSelectedItem();

        if (selectedConnection == null) {
            AlertHelper.warn(resources.getString("scriptStartWithoutConnectionNotPossibleTitle"),resources.getString("scriptStartWithoutConnectionNotPossibleContent"));
            return;
        }

        CorreoMqttClient client = ConnectionHolder.getInstance().getConnection(selectedConnection.getId()).getClient();
        if(client == null || client.getState() != CorreoMqttClientState.CONNECTED){
            AlertHelper.warn(resources.getString("scriptStartWithoutConnectedConnectionTitle"),resources.getString("scriptStartWithoutConnectedConnectionContent"));
            return;
        }

        scriptingStopButton.setDisable(false);
        scriptingRunButton.setDisable(true);
        statusText.setText(resources.getString("scriptExecutionRunning"));
        logArea.clear();

        TaskFactory.submitScript(ScriptExecutionDTO.builder()
                .jsCode(codeArea.getText())
                .out(out)
                .connectionId(selectedConnection.getId())
                .build());
    }

    @Override
    public void onSubmitScriptSucceeded(ScriptExecutionDTO scriptExecutionDTO) {
        // do nothing, wait for result
    }

    @Override
    public void onSubmitScriptCancelled(ScriptExecutionDTO scriptExecutionDTO) {

        scriptingStopButton.setDisable(true);
        scriptingRunButton.setDisable(false);
        statusText.setText(resources.getString("scriptCancelExecution"));

        try {
            scriptExecutionDTO.getOut().close();
        } catch (IOException e) {
            throw new CorreoMqttScriptExecutionFailed(e);
        }
    }

    @Override
    public void onSubmitScriptFailed(ScriptExecutionDTO scriptExecutionDTO, Throwable exception) {
        scriptingStopButton.setDisable(true);
        scriptingRunButton.setDisable(false);
        statusText.setText(resources.getString("scriptFailedExecution"));

        if(exception != null) {
            logArea.appendText(exception.getMessage());
        }else{
            logArea.appendText("Script Submission Failed.");
        }

        try {
            scriptExecutionDTO.getOut().close();
        } catch (IOException e) {
            throw new CorreoMqttScriptExecutionFailed(e);
        }
    }

    @Override
    public void onCancelScriptSucceeded(ScriptExecutionDTO scriptExecutionDTO) {
        // do nothing, wait for result
    }

    @Override
    public void onCancelScriptCancelled(ScriptExecutionDTO scriptExecutionDTO) {

    }

    @Override
    public void onCancelScriptFailed(ScriptExecutionDTO scriptExecutionDTO, Throwable exception) {

    }

    @Override
    public void onScriptExecutionSucceeded(ScriptExecutionDTO scriptExecutionDTO, long executionTimeInMilliseconds) {

        scriptingStopButton.setDisable(true);
        scriptingRunButton.setDisable(false);
        statusText.setText(MessageFormat.format(resources.getString("scriptSuccessExecution"), executionTimeInMilliseconds));

        try {
            scriptExecutionDTO.getOut().close();
        } catch (IOException e) {
            throw new CorreoMqttScriptExecutionFailed(e);
        }
    }

    @Override
    public void onScriptExecutionTimeout(ScriptExecutionDTO scriptExecutionDTO) {

    }

    @Override
    public void onScriptExecutionCancelled(ScriptExecutionDTO scriptExecutionDTO) {

    }

    @Override
    public void onScriptExecutionFailed(ScriptExecutionDTO scriptExecutionDTO, long executionTimeInMilliseconds, Throwable t) {

    }
}
