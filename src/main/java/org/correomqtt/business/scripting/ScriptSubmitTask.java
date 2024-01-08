package org.correomqtt.business.scripting;

import org.correomqtt.business.concurrent.Task;
import org.correomqtt.business.eventbus.EventBus;
import org.correomqtt.business.scripting.binding.ClientConnect;
import org.graalvm.polyglot.Context;
import org.graalvm.polyglot.PolyglotException;
import org.graalvm.polyglot.Value;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.slf4j.MarkerFactory;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.PipedInputStream;
import java.io.PipedOutputStream;
import java.nio.charset.StandardCharsets;
import java.text.MessageFormat;
import java.time.LocalDateTime;

public class ScriptSubmitTask extends Task<ExecutionDTO, ExecutionDTO, ExecutionDTO> {


    private static final Logger LOGGER = LoggerFactory.getLogger(ScriptSubmitTask.class);

    private final ExecutionDTO executionDTO;

    public ScriptSubmitTask(ExecutionDTO executionDTO) {
        this.executionDTO = executionDTO;
    }

    @FunctionalInterface
    public interface Executable {
        void onPromiseCreation(Value onResolve, Value onReject);
    }

    @Override
    protected ExecutionDTO execute() throws Exception {
        LOGGER.debug(MarkerFactory.getMarker(executionDTO.getScriptFile().getName()),
                "Submit script: {}", executionDTO.getExecutionId());

        ExecutionContextDTO executionContextDTO = ExecutionContextDTO.builder()
                .executionDTO(executionDTO)
                .build();

        executionDTO.setStartTime(LocalDateTime.now());
        Context context = null;

        try (PipedInputStream pis = new PipedInputStream();
             PipedOutputStream out = new PipedOutputStream(pis);
             ByteArrayOutputStream baos = new ByteArrayOutputStream(1024);
        ) {
            executionDTO.setIn(pis);
            context = new JsContextBuilder()
                    .dto(executionContextDTO)
                    .out(out)
                    .build();
            executionContextDTO.setContext(context);
            ScriptingBackend.getInstance().putExecution(executionDTO.getExecutionId(), executionContextDTO);
            EventBus.fireAsync(new ScriptExecutionProgressEvent(executionDTO));
            reportProgress(executionDTO);
            executeContext(context, executionContextDTO, executionDTO, out, baos);
            out.flush();
        } catch (Exception e) {
            executionDTO.updateExecutionTime();
            executionDTO.setError(new ScriptExecutionError(ScriptExecutionError.Type.HOST, e));
            if (LOGGER.isErrorEnabled()) {
                LOGGER.error(MarkerFactory.getMarker(executionDTO.getScriptFile().getName()),
                        "Script failed caused by host in {}ms. ",
                        executionDTO.getExecutionTime(), e);
            }
            EventBus.fireAsync(new ScriptExecutionFailedEvent(executionDTO));
            throw createExpectedException(executionDTO);
        } finally {
            if (context != null) {
                context.close();
            }
        }

        return executionDTO;
    }

    private void executeContext(Context context,
                                ExecutionContextDTO executionContextDTO,
                                ExecutionDTO executionDTO,
                                PipedOutputStream out, ByteArrayOutputStream baos) throws IOException {

        try {
            context.eval("js", executionDTO.getJsCode());
            executionDTO.updateExecutionTime();
            writeToOut(MessageFormat.format("[Correo] Script finished in {0}ms.",
                    executionDTO.getExecutionTime()), out, baos);
            if (LOGGER.isDebugEnabled()) {
                LOGGER.debug(MarkerFactory.getMarker(executionDTO.getScriptFile().getName()), "Script returned and succeeded in {}ms.",
                        executionDTO.getExecutionTime());
            }
            EventBus.fireAsync(new ScriptExecutionSuccessEvent(executionDTO));
        } catch (PolyglotException pge) {
            executionDTO.updateExecutionTime();
            handlePolyglotException(executionContextDTO, executionDTO, out, baos, pge);
            EventBus.fireAsync(new ScriptExecutionFailedEvent(executionDTO));
            throw createExpectedException(executionDTO);
        } catch (IOException e) {
            writeToOut(MessageFormat.format("[Correo] Script failed in {0}ms.",
                    executionDTO.getExecutionTime()), out, baos);
            throw e;
        }
    }


    private void handlePolyglotException(ExecutionContextDTO executionContextDTO,
                                         ExecutionDTO executionDTO,
                                         PipedOutputStream out,
                                         ByteArrayOutputStream baos,
                                         PolyglotException pge) throws IOException {
        try {
            if (pge.isGuestException()) {
                executionDTO.setError(new ScriptExecutionError(ScriptExecutionError.Type.GUEST, pge));
                if (pge.isCancelled()) {
                    executionDTO.setCancelled(true);
                    writeToOut(MessageFormat.format("[Correo] Script cancelled after {0}ms.",
                            executionDTO.getExecutionTime()), out, baos);
                    if (LOGGER.isInfoEnabled()) {
                        LOGGER.info(MarkerFactory.getMarker(executionDTO.getScriptFile().getName()), "Script cancelled after {}ms.\n{}",
                                executionDTO.getExecutionTime(), pge.getMessage());
                    }
                } else {
                    writeToOut(MessageFormat.format("[Correo] Script failed in {0}ms. {1}",
                            executionDTO.getExecutionTime(), pge.getMessage()), out, baos);
                    if (LOGGER.isWarnEnabled()) {
                        LOGGER.warn(MarkerFactory.getMarker(executionDTO.getScriptFile().getName()), "Script failed due to javascript error in {}ms.\n{}", executionDTO.getExecutionTime(), pge.getMessage());
                    }
                }
            } else {
                writeToOut(MessageFormat.format("[Correo] Script failed in {0}ms. {1}",
                        executionDTO.getExecutionTime(), pge.getMessage()), out, baos);
                executionDTO.setError(new ScriptExecutionError(ScriptExecutionError.Type.HOST, pge));
                if (LOGGER.isWarnEnabled()) {
                    LOGGER.error(MarkerFactory.getMarker(executionDTO.getScriptFile().getName()), "Script failed caused by host in {}ms. ",
                            executionDTO.getExecutionTime(), pge);
                }
            }
        } catch (IOException e) {
            writeToOut(MessageFormat.format("[Correo] Script failed in {0}ms",
                    executionDTO.getExecutionTime()), out, baos);
            throw e;
        }
    }

    private void writeToOut(String msg, PipedOutputStream out, ByteArrayOutputStream baos) throws IOException {
/*        baos.write(msg.getBytes(StandardCharsets.UTF_8));
        baos.writeTo(out);
        baos.flush();

 */
        out.write(msg.getBytes(StandardCharsets.UTF_8));
        out.flush();
    }
}
