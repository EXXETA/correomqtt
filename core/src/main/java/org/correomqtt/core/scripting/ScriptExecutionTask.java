package org.correomqtt.core.scripting;

import lombok.Getter;
import org.correomqtt.di.Assisted;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import org.correomqtt.di.Lazy;
import org.correomqtt.core.concurrent.FullTask;
import org.correomqtt.core.concurrent.TaskException;
import org.correomqtt.core.eventbus.EventBus;
import org.correomqtt.core.fileprovider.ScriptingProvider;
import org.graalvm.polyglot.Context;
import org.graalvm.polyglot.PolyglotException;
import org.graalvm.polyglot.Source;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.slf4j.Marker;

import java.io.PipedOutputStream;
import java.time.LocalDateTime;

import static org.correomqtt.core.scripting.ScriptExecutionError.Type.GUEST;
import static org.correomqtt.core.scripting.ScriptExecutionError.Type.HOST;
import static org.slf4j.MarkerFactory.getMarker;

@DefaultBean
public class ScriptExecutionTask extends FullTask<ExecutionDTO, ExecutionDTO, ExecutionDTO> {

    private static final Logger LOGGER = LoggerFactory.getLogger(ScriptExecutionTask.class);
    private final ScriptLoggerContextFactory scriptLoggerContextFactory;
    private final ScriptingProvider scriptingProvider;
    private final Lazy<JsContextBuilder> jsContextBuilderLazy;
    @Getter
    private final ExecutionDTO dto;
    private Context context;

    @Inject
    public ScriptExecutionTask(
            ScriptLoggerContextFactory scriptLoggerContextFactory,
            ScriptingProvider scriptingProvider,
            Lazy<JsContextBuilder> jsContextBuilderLazy,
            EventBus eventBus,
            @Assisted ExecutionDTO executionDTO) {
        super(eventBus);
        this.scriptLoggerContextFactory = scriptLoggerContextFactory;
        this.scriptingProvider = scriptingProvider;
        this.jsContextBuilderLazy = jsContextBuilderLazy;
        this.dto = executionDTO;
    }

    public ExecutionDTO execute() throws TaskException {
        LOGGER.debug(marker(), "Submit script: {}", dto.getExecutionId());
        ScriptingBackend.putExecutionTask(this);
        try (ScriptLoggerContext slc = scriptLoggerContextFactory.create(dto, marker())) {
            dto.setStartTime(LocalDateTime.now());
            scriptingProvider.saveExecution(dto);
            ch.qos.logback.classic.Logger scriptLogger = slc.getScriptLogger();
            dto.setLogger(scriptLogger);
            executeJs(slc, scriptLogger);
            scriptingProvider.saveExecution(dto);
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            LOGGER.info(marker(), "Exception during Script execution", e);
            dto.updateExecutionTime();
            dto.setError(new ScriptExecutionError(HOST, e.getMessage()));
            eventBus.fire(new ScriptExecutionFailedEvent(dto));
        } catch (Exception e) {
            LOGGER.info(marker(), "Exception during Script execution", e);
            dto.updateExecutionTime();
            dto.setError(new ScriptExecutionError(HOST, e.getMessage()));
            eventBus.fire(new ScriptExecutionFailedEvent(dto));
        }
        return dto;
    }

    private Marker marker() {
        return getMarker(dto.getScriptFile().getName());
    }

    private void executeJs(ScriptLoggerContext slc, ch.qos.logback.classic.Logger scriptLogger) {
        try (PipedOutputStream out = new PipedOutputStream();
             Context c = jsContextBuilderLazy.get()
                     .dto(dto)
                     .out(out)
                     .marker(marker())
                     .logger(slc.getScriptLogger())
                     .build()) {
            context = c;
            slc.connectSnk(out);
            eventBus.fire(new ScriptExecutionProgressEvent(dto));
            reportProgress(dto);
            Source source = Source.newBuilder("js",
                            "logger.info(marker,\"Running script with ECMAScript {} on GraalJS.\", Graal.versionECMAScript);\n" +
                                    dto.getJsCode() +
                                    "\njoin();", "jscode")
                    .mimeType("application/javascript+module") // required for top level await
                    .build();
            context.eval(source);
            dto.updateExecutionTime();
            scriptLogger.info(marker(), "Script returned in {}ms.", dto.getExecutionTime());
            eventBus.fire(new ScriptExecutionSuccessEvent(dto));
        } catch (PolyglotException pge) {
            dto.updateExecutionTime();
            ScriptExecutionError.Type type = pge.isGuestException() ? GUEST : HOST;
            dto.setError(new ScriptExecutionError(type, pge.getMessage()));
            if (pge.isCancelled()) {
                dto.setCancelled(true);
                scriptLogger.info(marker(), "Script cancelled after {}ms by {}.\n{}", dto.getExecutionTime(), type, pge.getMessage());
                eventBus.fire(new ScriptExecutionCancelledEvent(dto));
            } else {
                scriptLogger.error(marker(), "Script failed after {}ms by {}.\n{}", dto.getExecutionTime(), type, pge.getMessage());
                eventBus.fire(new ScriptExecutionFailedEvent(dto));
            }
        } catch (Exception e) {
            LOGGER.info(marker(), "Exception during Script execution", e);
            dto.updateExecutionTime();
            dto.setError(new ScriptExecutionError(HOST, e.getMessage()));
            eventBus.fire(new ScriptExecutionFailedEvent(dto));
        }
    }

    public void cancel() {
        dto.setCancelled(true);
        this.context.close(true);
    }
}
