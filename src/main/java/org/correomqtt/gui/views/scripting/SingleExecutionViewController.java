package org.correomqtt.gui.views.scripting;

import ch.qos.logback.classic.LoggerContext;
import ch.qos.logback.classic.spi.ILoggingEvent;
import ch.qos.logback.core.encoder.Encoder;
import javafx.fxml.FXML;
import javafx.scene.control.Button;
import javafx.scene.layout.Pane;
import javafx.scene.layout.VBox;
import org.correomqtt.business.eventbus.EventBus;
import org.correomqtt.business.eventbus.Subscribe;
import org.correomqtt.business.eventbus.SubscribeFilter;
import org.correomqtt.business.scripting.ExecutionDTO;
import org.correomqtt.business.scripting.ScriptCancelTask;
import org.correomqtt.business.scripting.ScriptExecutionCancelledEvent;
import org.correomqtt.business.scripting.ScriptExecutionFailedEvent;
import org.correomqtt.business.scripting.ScriptExecutionProgressEvent;
import org.correomqtt.business.scripting.ScriptExecutionSuccessEvent;
import org.correomqtt.business.scripting.ScriptLoadLogTask;
import org.correomqtt.business.scripting.ScriptingBackend;
import org.correomqtt.gui.log.LogToRichtTextFxAppender;
import org.correomqtt.gui.utils.AlertHelper;
import org.correomqtt.gui.utils.LogAreaUtils;
import org.correomqtt.gui.views.LoaderResult;
import org.correomqtt.gui.views.base.BaseControllerImpl;
import org.fxmisc.richtext.CodeArea;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.ResourceBundle;

import static org.correomqtt.business.eventbus.SubscribeFilterNames.SCRIPT_EXECUTION_ID;
import static org.correomqtt.business.utils.LoggerUtils.SCRIPT_COLOR_PATTERN_APPENDER_NAME;
import static org.correomqtt.business.utils.LoggerUtils.findPatternEncoder;

public class SingleExecutionViewController extends BaseControllerImpl {

    private static final Logger LOGGER = LoggerFactory.getLogger(SingleExecutionViewController.class);
    private static ResourceBundle resources;
    @FXML
    private VBox mainNode;
    @FXML
    private Pane logHolder;
    @FXML
    private CodeArea logArea;
    @FXML
    private Button scriptingStopButton;
    private final ExecutionPropertiesDTO executionPropertiesDTO;
    private LogToRichtTextFxAppender appender;

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
    private void initialize() {
        logArea.prefWidthProperty().bind(logHolder.widthProperty());
        logArea.prefHeightProperty().bind(logHolder.heightProperty());
        ExecutionDTO dto = ScriptingBackend.getExecutionDTO(executionPropertiesDTO.getExecutionId());

        if (dto != null) {
            new ScriptLoadLogTask(dto)
                    .onSuccess(log -> {
                        LogAreaUtils.appendColorful(logArea, log);
                        if (executionPropertiesDTO.getState() == ScriptState.RUNNING) {
                            scriptingStopButton.setDisable(false);
                            connectLog(dto);
                        } else {
                            scriptingStopButton.setDisable(true);
                        }
                    })
                    .onError(error -> AlertHelper.unexpectedAlert(error.getUnexpectedError()))
                    .run();
        }
    }

    @SuppressWarnings("unused")
    public void onScriptExecutionProgress(@Subscribe(sync = true) ScriptExecutionProgressEvent event) {
        connectLog(event.getExecutionDTO());
    }

    private void connectLog(ExecutionDTO dto) {
        if (appender != null) {
            return;
        }

        scriptingStopButton.setDisable(false);

        LoggerContext context = (LoggerContext) LoggerFactory.getILoggerFactory();

        Encoder<ILoggingEvent> encoder = findPatternEncoder(SCRIPT_COLOR_PATTERN_APPENDER_NAME);

        appender = new LogToRichtTextFxAppender(logArea);
        appender.setName(executionPropertiesDTO.getExecutionId());
        appender.setContext(context);
        appender.setEncoder(encoder);
        appender.start();

        ch.qos.logback.classic.Logger scriptLogger = dto.getLogger();
        //noinspection java:S4792
        scriptLogger.addAppender(appender);

    }

    @SuppressWarnings("unused")
    public void onScriptExecutionCancelled(@Subscribe ScriptExecutionCancelledEvent event) {
        scriptingStopButton.setDisable(true);
        disconnectLog(event.getExecutionDTO());
    }

    @SuppressWarnings("unused")
    public void onScriptExecutionSuccess(@Subscribe ScriptExecutionSuccessEvent event) {
        scriptingStopButton.setDisable(true);
        disconnectLog(event.getExecutionDTO());
    }

    @SuppressWarnings("unused")
    public void onScriptExecutionFailed(@Subscribe ScriptExecutionFailedEvent event) {
        scriptingStopButton.setDisable(true);
        disconnectLog(event.getExecutionDTO());
    }

    private void disconnectLog(ExecutionDTO dto) {
        ch.qos.logback.classic.Logger scriptLogger = dto.getLogger();
        if (scriptLogger != null) {
            scriptLogger.detachAppender(appender);
        }
    }


    @SuppressWarnings("unused")
    @SubscribeFilter(SCRIPT_EXECUTION_ID)
    public String getExecutionId() {
        return executionPropertiesDTO.getExecutionId();
    }

    public void cleanup() {
        if (appender != null) {
            appender.stop();
        }
        EventBus.unregister(this);
    }

    @FXML
    private void onStopButtonClicked() {
        new ScriptCancelTask(getExecutionId())
                .onError(error -> AlertHelper.unexpectedAlert(error.getUnexpectedError()))
                .run();
    }

}
