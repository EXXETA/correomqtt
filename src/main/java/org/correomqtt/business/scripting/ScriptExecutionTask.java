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

import static java.text.MessageFormat.format;
import static org.correomqtt.business.scripting.ScriptExecutionError.Type.GUEST;
import static org.correomqtt.business.scripting.ScriptExecutionError.Type.HOST;
import static org.slf4j.MarkerFactory.getMarker;

public class ScriptExecutionTask extends Task<ExecutionDTO, ExecutionDTO, ExecutionDTO> {


    private static final Logger LOGGER = LoggerFactory.getLogger(ScriptExecutionTask.class);
    private static final int MAX_WAIT = 5000;
    private static final int WAIT_STEP = 100;
    private PipedInputStream snk = null;
    @Getter
    private final ExecutionDTO dto;
    private Context context;

    public ScriptExecutionTask(ExecutionDTO executionDTO) {
        this.dto = executionDTO;
    }

    public ExecutionDTO execute() throws TaskException {
        LOGGER.debug(getMarker(dto.getScriptFile().getName()),
                "Submit script: {}", dto.getExecutionId());

        ScriptingBackend.putExecutionTask(dto.getExecutionId(), this);

        try (PipedOutputStream out = new PipedOutputStream();
             ScriptLoggerContext slc = new ScriptLoggerContext(out, dto.getScriptFile().getName());
             Context c = new JsContextBuilder()
                     .dto(dto)
                     .out(out)
                     .logger(slc.getLogger())
                     .build();
        ) {

            context = c;

            Logger scriptLogger = slc.getLogger();

            dto.setStartTime(LocalDateTime.now());

            dto.setConnectSnk(incSnk -> this.connectSnkToOut(incSnk, out));

            EventBus.fireAsync(new ScriptExecutionProgressEvent(dto));
            reportProgress(dto);

            executeContext(context, scriptLogger);

            dto.updateExecutionTime();

            outlog(scriptLogger, format("Script returned and succeeded in {0}ms.", dto.getExecutionTime()));

            waitForSnk();

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

    private void executeContext(Context context, Logger scriptLogger) throws IOException {
        try {
            //context.eval("js", dto.getJsCode());
            Source source = Source.newBuilder("js", dto.getJsCode(), "test")
                    .mimeType("application/javascript+module").build();
            context.eval(source);
        } catch (PolyglotException pge) {
            outlog(scriptLogger, pge.getMessage());
            dto.updateExecutionTime();
            handlePolyglotException(pge, scriptLogger);
            throw new TaskException(dto);
        }
    }

    private void handlePolyglotException(PolyglotException pge, Logger scriptLogger) {

        if (pge.isGuestException()) {
            dto.setError(new ScriptExecutionError(GUEST, pge));
            if (pge.isCancelled()) {
                dto.setCancelled(true);
                outlog(scriptLogger, format("Script cancelled after {0}ms.\n{1}", dto.getExecutionTime(), pge.getMessage()));
            } else {
                outlog(scriptLogger, format("Script failed after {0}ms.\n{1}", dto.getExecutionTime(), pge.getMessage()));
            }
        } else {
            dto.setError(new ScriptExecutionError(HOST, pge));
            outlog(scriptLogger, format("Script failed caused by host after {0}ms.\n{1}", dto.getExecutionTime(), pge.getMessage()));
        }
    }

    private void waitForSnk() throws IOException, InterruptedException {
        if (snk == null)
            return;

        for (int currentWait = 0; currentWait < MAX_WAIT; currentWait += WAIT_STEP) {
            if (snk.available() <= 0) {
                break;
            }
            Thread.sleep(WAIT_STEP);
        }

        if (LOGGER.isDebugEnabled() && snk.available() > 0)
            LOGGER.debug("SNK still has data available after {}ms => Stopping now.", MAX_WAIT);
    }

    private void outlog(Logger scriptLogger, String msg) {
        scriptLogger.info(msg);
        if (LOGGER.isInfoEnabled()) {
            LOGGER.info(marker(), msg);
        }
    }

    private Marker marker() {
        return getMarker(dto.getScriptFile().getName());
    }

    private void connectSnkToOut(PipedInputStream snk, PipedOutputStream out) throws TaskException {
        this.snk = snk;
        try {
            out.connect(snk);
        } catch (IOException e) {
            LOGGER.error("Unable to connect out to snk", e);
            throw new TaskException(dto);
        }
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
