package org.correomqtt.gui.views.scripting;

import ch.qos.logback.classic.LoggerContext;
import ch.qos.logback.classic.spi.ILoggingEvent;
import ch.qos.logback.core.encoder.Encoder;
import dagger.assisted.Assisted;
import dagger.assisted.AssistedInject;
import javafx.fxml.FXML;
import javafx.scene.control.Button;
import javafx.scene.layout.Pane;
import javafx.scene.layout.VBox;
import org.correomqtt.core.settings.SettingsProvider;
import org.correomqtt.core.eventbus.EventBus;
import org.correomqtt.core.eventbus.Subscribe;
import org.correomqtt.core.eventbus.SubscribeFilter;
import org.correomqtt.core.scripting.ExecutionDTO;
import org.correomqtt.core.scripting.ScriptCancelTask;
import org.correomqtt.core.scripting.ScriptExecutionCancelledEvent;
import org.correomqtt.core.scripting.ScriptExecutionFailedEvent;
import org.correomqtt.core.scripting.ScriptExecutionProgressEvent;
import org.correomqtt.core.scripting.ScriptExecutionSuccessEvent;
import org.correomqtt.core.scripting.ScriptLoadLogTask;
import org.correomqtt.core.scripting.ScriptingBackend;
import org.correomqtt.gui.log.LogToRichtTextFxAppender;
import org.correomqtt.gui.theme.ThemeManager;
import org.correomqtt.gui.utils.AlertHelper;
import org.correomqtt.gui.utils.LogAreaUtils;
import org.correomqtt.gui.views.LoaderResult;
import org.correomqtt.gui.views.base.BaseControllerImpl;
import org.fxmisc.richtext.CodeArea;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.ResourceBundle;

import static org.correomqtt.core.eventbus.SubscribeFilterNames.SCRIPT_EXECUTION_ID;
import static org.correomqtt.core.utils.LoggerUtils.SCRIPT_COLOR_PATTERN_APPENDER_NAME;
import static org.correomqtt.core.utils.LoggerUtils.findPatternEncoder;

public class SingleExecutionViewController extends BaseControllerImpl {

    private static final Logger LOGGER = LoggerFactory.getLogger(SingleExecutionViewController.class);
    private static ResourceBundle resources;
    private final AlertHelper alertHelper;
    private final ExecutionPropertiesDTO executionPropertiesDTO;
    @FXML
    private VBox mainNode;
    @FXML
    private Pane logHolder;
    @FXML
    private CodeArea logArea;
    @FXML
    private Button scriptingStopButton;
    private LogToRichtTextFxAppender appender;

    @AssistedInject
    public SingleExecutionViewController(SettingsProvider settingsProvider,
                                         ThemeManager themeManager,
                                         AlertHelper alertHelper,
                                         @Assisted ExecutionPropertiesDTO executionPropertiesDTO) {
        super(settingsProvider, themeManager);
        this.alertHelper = alertHelper;
        this.executionPropertiesDTO = executionPropertiesDTO;
        EventBus.register(this);
    }

    public LoaderResult<SingleExecutionViewController> load() {

        LoaderResult<SingleExecutionViewController> result = load(SingleExecutionViewController.class, "singleExecutionView.fxml",
                () -> this);
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
                    .onError(error -> alertHelper.unexpectedAlert(error.getUnexpectedError()))
                    .run();
        }
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
    public void onScriptExecutionProgress(@Subscribe(sync = true) ScriptExecutionProgressEvent event) {
        connectLog(event.getExecutionDTO());
    }

    @SuppressWarnings("unused")
    public void onScriptExecutionCancelled(@Subscribe ScriptExecutionCancelledEvent event) {
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
    public void onScriptExecutionSuccess(@Subscribe ScriptExecutionSuccessEvent event) {
        scriptingStopButton.setDisable(true);
        disconnectLog(event.getExecutionDTO());
    }

    @SuppressWarnings("unused")
    public void onScriptExecutionFailed(@Subscribe ScriptExecutionFailedEvent event) {
        scriptingStopButton.setDisable(true);
        disconnectLog(event.getExecutionDTO());
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
                .onError(error -> alertHelper.unexpectedAlert(error.getUnexpectedError()))
                .run();
    }

    @SuppressWarnings("unused")
    @SubscribeFilter(SCRIPT_EXECUTION_ID)
    public String getExecutionId() {
        return executionPropertiesDTO.getExecutionId();
    }

}
