package org.correomqtt.business.scripting;

import lombok.Getter;
import org.correomqtt.business.concurrent.Task;
import org.correomqtt.business.concurrent.TaskErrorResult;
import org.correomqtt.business.concurrent.TaskException;
import org.correomqtt.business.eventbus.EventBus;
import org.graalvm.polyglot.Context;
import org.graalvm.polyglot.PolyglotException;
import org.graalvm.polyglot.Source;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.slf4j.Marker;

import java.io.IOException;
import java.io.PipedInputStream;
import java.io.PipedOutputStream;
import java.time.LocalDateTime;

import static org.correomqtt.business.scripting.ScriptExecutionError.Type.GUEST;
import static org.correomqtt.business.scripting.ScriptExecutionError.Type.HOST;
import static org.slf4j.MarkerFactory.getMarker;

public class ScriptExecutionTask extends Task<ExecutionDTO, ExecutionDTO, ExecutionDTO> {

    private static final Logger LOGGER = LoggerFactory.getLogger(ScriptExecutionTask.class);
    private static final int MAX_WAIT = 5000;
    private static final int WAIT_STEP = 100;
    @Getter
    private final ExecutionDTO dto;
    private Context context;

    public ScriptExecutionTask(ExecutionDTO executionDTO) {
        this.dto = executionDTO;
    }

    public ExecutionDTO execute() throws TaskException {
        LOGGER.debug(getMarker(dto.getScriptFile().getName()), "Submit script: {}", dto.getExecutionId());

        ScriptingBackend.putExecutionTask(dto.getExecutionId(), this);

        try (PipedOutputStream out = new PipedOutputStream();
             PipedInputStream snk = new PipedInputStream(out);
             ScriptLoggerContext slc = new ScriptLoggerContext(out, dto.getExecutionId(), dto.getScriptFile().getName());
             Context c = new JsContextBuilder().dto(dto).out(out).logger(slc.getScriptLogger()).build();) {
            context = c;
            Logger scriptLogger = slc.getScriptLogger();
            dto.setStartTime(LocalDateTime.now());
            dto.setSnk(snk);
            EventBus.fireAsync(new ScriptExecutionProgressEvent(dto));
            reportProgress(dto);
            if (!executeContext(context, scriptLogger)) {
                throw new TaskException(dto);
            }
            dto.updateExecutionTime();
            scriptLogger.info("Script returned in {}ms.", dto.getExecutionTime());
            waitForSnk(snk);
            EventBus.fire(new ScriptExecutionSuccessEvent(dto));
            return dto;
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            LOGGER.info("InterruptedException during Script execution", e);
            dto.updateExecutionTime();
            dto.setError(new ScriptExecutionError(HOST, e));
            throw new TaskException(dto);
        } catch (Exception e) {
            LOGGER.info("Exception during Script execution", e);
            dto.updateExecutionTime();
            dto.setError(new ScriptExecutionError(HOST, e));
            throw new TaskException(dto);
        }
    }

    private boolean executeContext(Context context, Logger scriptLogger) throws IOException {
        try {
            Source source = Source.newBuilder("js", dto.getJsCode() + "\njoin();", "test")
                    .mimeType("application/javascript+module") // required for top level await
                    .build();
            context.eval(source);
            return true;
        } catch (PolyglotException pge) {
            return handlePolyglotException(pge, scriptLogger);
        }
    }

    private boolean handlePolyglotException(PolyglotException pge, Logger scriptLogger) {
        dto.updateExecutionTime();
        if (pge.isGuestException()) {
            dto.setError(new ScriptExecutionError(GUEST, pge));
            if (pge.isCancelled()) {
                dto.setCancelled(true);
                scriptLogger.info("Script cancelled after {}ms.\n{}", dto.getExecutionTime(), pge.getMessage());
                return true;
            } else {
                scriptLogger.error("Script failed after {}ms.\n{}", dto.getExecutionTime(), pge.getMessage());
                return false;
            }
        } else {
            dto.setError(new ScriptExecutionError(HOST, pge));
            scriptLogger.error("Script failed caused by host after {}ms.\n{}", dto.getExecutionTime(), pge.getMessage());
            return false;
        }
    }

    private void waitForSnk(PipedInputStream snk) throws IOException, InterruptedException {
        if (snk == null) return;

        for (int currentWait = 0; currentWait < MAX_WAIT; currentWait += WAIT_STEP) {
            if (snk.available() <= 0) {
                break;
            }
            Thread.sleep(WAIT_STEP);
        }

        if (LOGGER.isDebugEnabled() && snk.available() > 0)
            LOGGER.debug("SNK still has data available after {}ms => Stopping now.", MAX_WAIT);
    }

    private Marker marker() {
        return getMarker(dto.getScriptFile().getName());
    }


    public void cancel() {
        dto.setCancelled(true);
        this.context.close(true);
    }

    @Override
    protected void errorHook(TaskErrorResult<ExecutionDTO> errorResult) {
        EventBus.fire(new ScriptExecutionFailedEvent(errorResult.getExpectedError()));
    }


}
