package org.correomqtt.gui.views.scripting;

import javafx.application.Platform;
import javafx.event.ActionEvent;
import javafx.fxml.FXML;
import javafx.scene.control.Button;
import javafx.scene.layout.Pane;
import javafx.scene.layout.VBox;
import org.correomqtt.business.connection.ConnectionStateChangedEvent;
import org.correomqtt.business.eventbus.EventBus;
import org.correomqtt.business.eventbus.Subscribe;
import org.correomqtt.business.eventbus.SubscribeFilter;
import org.correomqtt.business.scripting.ExecutionDTO;
import org.correomqtt.business.scripting.ScriptCancelTask;
import org.correomqtt.business.scripting.ScriptExecutionError;
import org.correomqtt.gui.views.LoaderResult;
import org.correomqtt.gui.views.base.BaseControllerImpl;
import org.fxmisc.richtext.CodeArea;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.nio.charset.StandardCharsets;
import java.text.MessageFormat;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;

import static org.correomqtt.business.eventbus.SubscribeFilterNames.SCRIPT_EXECUTION_ID;

public class SingleExecutionViewController extends BaseControllerImpl {

    private static final Logger LOGGER = LoggerFactory.getLogger(SingleExecutionViewController.class);
    private static ResourceBundle resources;
    @FXML
    public VBox mainNode;
    @FXML
    public Pane logHolder;
    @FXML
    private CodeArea logArea;
    @FXML
    private Button scriptingStopButton;
    private final ExecutionPropertiesDTO executionPropertiesDTO;
    private CompletableFuture<Void> logPipeFuture;

    public SingleExecutionViewController(ExecutionPropertiesDTO executionPropertiesDTO) {
        this.executionPropertiesDTO = executionPropertiesDTO;
        EventBus.register(this);
    }

    public static LoaderResult<SingleExecutionViewController> load(ExecutionPropertiesDTO executionPropertiesDTO) {

        LoaderResult<SingleExecutionViewController> result = load(SingleExecutionViewController.class, "singleExecutionView.fxml",
                () -> new SingleExecutionViewController(executionPropertiesDTO));
        resources = result.getResourceBundle();
        return result;
    }

    @FXML
    public void initialize() {
        logArea.prefWidthProperty().bind(logHolder.widthProperty());
        logArea.prefHeightProperty().bind(logHolder.heightProperty());
        scriptingStopButton.setDisable(true);
    }

    public void onScriptExecutionProgress(ExecutionDTO dto) {

        if (logPipeFuture != null) {
            return;
        }

        scriptingStopButton.setDisable(false);

        logPipeFuture = CompletableFuture.runAsync(() -> {
                    final int BUFFER_SIZE = 8192;

                    try (final InputStreamReader isr = new InputStreamReader(dto.getIn(), StandardCharsets.UTF_8);
                         final BufferedReader br = new BufferedReader(isr, BUFFER_SIZE);
                    ) {
                        String line;
                        while ((line = br.readLine()) != null) {
                            final String text = line;
                            Platform.runLater(() -> addLog(text));
                        }

                    } catch (IOException e) {
                        LOGGER.debug("Pipe to script log broke.");
                    }
                }).

                exceptionallyAsync(e -> {
                    LOGGER.error("Exception listening to script pipe. ", e);
                    return null;
                });
    }



    private void addLog(String msg) {
        this.logArea.append(msg + "\n", "default");
        this.logArea.requestFollowCaret();
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("[{}] {} ", getExecutionId(), msg);
        }
    }

    @SuppressWarnings("unused")
    public void onScriptExecutionSuccess(ExecutionDTO dto) {
        scriptingStopButton.setDisable(true);
    }

    public void onScriptExecutionFailed(ExecutionDTO dto) {
        scriptingStopButton.setDisable(true);
    }

    @SuppressWarnings("unused")
    @SubscribeFilter(SCRIPT_EXECUTION_ID)
    public String getExecutionId() {
        return executionPropertiesDTO.getExecutionId();
    }

    public void onCloseRequest() {
        EventBus.unregister(this);
    }

    @FXML
    public void onStopButtonClicked() {
       new ScriptCancelTask(getExecutionId()).run();
    }

}
