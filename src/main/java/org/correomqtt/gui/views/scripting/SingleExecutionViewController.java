package org.correomqtt.gui.views.scripting;

import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.scene.control.Button;
import javafx.scene.layout.Pane;
import javafx.scene.layout.VBox;
import org.correomqtt.business.eventbus.EventBus;
import org.correomqtt.business.eventbus.Subscribe;
import org.correomqtt.business.eventbus.SubscribeFilter;
import org.correomqtt.business.scripting.ScriptCancelTask;
import org.correomqtt.business.scripting.ScriptExecutionFailedEvent;
import org.correomqtt.business.scripting.ScriptExecutionProgressEvent;
import org.correomqtt.business.scripting.ScriptExecutionSuccessEvent;
import org.correomqtt.gui.utils.LogAreaUtils;
import org.correomqtt.gui.views.LoaderResult;
import org.correomqtt.gui.views.base.BaseControllerImpl;
import org.fxmisc.richtext.CodeArea;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.io.PipedInputStream;
import java.nio.charset.StandardCharsets;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.TimeUnit;

import static org.correomqtt.business.eventbus.SubscribeFilterNames.SCRIPT_EXECUTION_ID;

public class SingleExecutionViewController extends BaseControllerImpl {

    private static final Logger LOGGER = LoggerFactory.getLogger(SingleExecutionViewController.class);
    private static final int MAX_WAIT_FOR_SNK_CONNECTED = 5000;
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

    @SuppressWarnings("unused")
    public void onScriptExecutionProgress(@Subscribe(sync = true) ScriptExecutionProgressEvent event) {

        if (logPipeFuture != null) {
            return;
        }

        scriptingStopButton.setDisable(false);

        // Barrier to wait till log is connected before script is started
        CountDownLatch pisConnected = new CountDownLatch(1);

        logPipeFuture = CompletableFuture.runAsync(() -> {
            final int BUFFER_SIZE = 8192;
            try (final PipedInputStream snk = new PipedInputStream();
                 final InputStreamReader isr = new InputStreamReader(snk, StandardCharsets.UTF_8);
                 final BufferedReader br = new BufferedReader(isr, BUFFER_SIZE);
            ) {

                // connect snk
                event.getExecutionDTO().getConnectSnk().accept(snk);
                pisConnected.countDown();

                // stream log output
                String line;
                while ((line = br.readLine()) != null) {
                    final String text = line;
                    Platform.runLater(() -> addLog(text));
                }

            } catch (IOException e) {
                // this is normal if SNK is closed.
                LOGGER.trace("Pipe to script log broke.", e);
            }
        }).exceptionallyAsync(e -> {
            LOGGER.error("Exception listening to script pipe. ", e);
            return null;
        });

        try {
            if (!pisConnected.await(MAX_WAIT_FOR_SNK_CONNECTED, TimeUnit.MILLISECONDS)) {
                throw new IllegalStateException("Snk not connected in " + MAX_WAIT_FOR_SNK_CONNECTED + "ms.");
            }
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            throw new IllegalStateException(e);
        }
    }

    private void addLog(String msg) {
        LogAreaUtils.appendColorful(logArea, msg + "\n");
        logArea.requestFollowCaret();
        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("[{}] {} ", getExecutionId(), msg);
        }
    }

    @SuppressWarnings("unused")
    @Subscribe(ScriptExecutionSuccessEvent.class)
    public void onScriptExecutionSuccess() {

        scriptingStopButton.setDisable(true);
    }

    @SuppressWarnings("unused")
    @Subscribe(ScriptExecutionFailedEvent.class)

    public void onScriptExecutionFailed() {
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
        new ScriptCancelTask(getExecutionId())
                .run();
    }

}
