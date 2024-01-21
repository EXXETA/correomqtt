package org.correomqtt.core.scripting;

import dagger.Lazy;
import dagger.assisted.Assisted;
import dagger.assisted.AssistedInject;
import lombok.Getter;
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

public class ScriptExecutionTask extends FullTask<ExecutionDTO, ExecutionDTO, ExecutionDTO> {

    private static final Logger LOGGER = LoggerFactory.getLogger(ScriptExecutionTask.class);
    private final Lazy<JsContextBuilder> jsContextBuilderLazy;
    @Getter
    private final ExecutionDTO dto;
    private Context context;

    @AssistedInject
    public ScriptExecutionTask(
            Lazy<JsContextBuilder> jsContextBuilderLazy,
            @Assisted ExecutionDTO executionDTO) {
        this.jsContextBuilderLazy = jsContextBuilderLazy;
        this.dto = executionDTO;
    }

    public ExecutionDTO execute() throws TaskException {
        LOGGER.debug(marker(), "Submit script: {}", dto.getExecutionId());

        ScriptingBackend.putExecutionTask(this);
        try (ScriptLoggerContext slc = new ScriptLoggerContext(dto, marker())) {
            dto.setStartTime(LocalDateTime.now());
            ScriptingProvider.getInstance().saveExecution(dto);
            ch.qos.logback.classic.Logger scriptLogger = slc.getScriptLogger();
            dto.setLogger(scriptLogger);
            executeJs(slc, scriptLogger);
            ScriptingProvider.getInstance().saveExecution(dto);
        }catch(InterruptedException e){
            Thread.currentThread().interrupt();
            LOGGER.info(marker(), "Exception during Script execution", e);
            dto.updateExecutionTime();
            dto.setError(new ScriptExecutionError(HOST, e.getMessage()));
            EventBus.fire(new ScriptExecutionFailedEvent(dto));
        } catch (Exception e) {
            LOGGER.info(marker(), "Exception during Script execution", e);
            dto.updateExecutionTime();
            dto.setError(new ScriptExecutionError(HOST, e.getMessage()));
            EventBus.fire(new ScriptExecutionFailedEvent(dto));
        }
        return dto;
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
            EventBus.fire(new ScriptExecutionProgressEvent(dto));
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
            EventBus.fire(new ScriptExecutionSuccessEvent(dto));
        } catch (PolyglotException pge) {
            dto.updateExecutionTime();
            ScriptExecutionError.Type type = pge.isGuestException() ? GUEST : HOST;
            dto.setError(new ScriptExecutionError(type, pge.getMessage()));
            if (pge.isCancelled()) {
                dto.setCancelled(true);
                scriptLogger.info(marker(), "Script cancelled after {}ms by {}.\n{}", dto.getExecutionTime(), type, pge.getMessage());
                EventBus.fire(new ScriptExecutionCancelledEvent(dto));
            } else {
                scriptLogger.error(marker(), "Script failed after {}ms by {}.\n{}", dto.getExecutionTime(), type, pge.getMessage());
                EventBus.fire(new ScriptExecutionFailedEvent(dto));
            }
        } catch (Exception e) {
            LOGGER.info(marker(), "Exception during Script execution", e);
            dto.updateExecutionTime();
            dto.setError(new ScriptExecutionError(HOST, e.getMessage()));
            EventBus.fire(new ScriptExecutionFailedEvent(dto));
        }
    }

    private Marker marker() {
        return getMarker(dto.getScriptFile().getName());
    }

    public void cancel() {
        dto.setCancelled(true);
        this.context.close(true);
    }
}
