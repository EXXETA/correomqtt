package org.correomqtt.business.scripting;

import ch.qos.logback.classic.Logger;
import ch.qos.logback.classic.LoggerContext;
import ch.qos.logback.classic.spi.ILoggingEvent;
import ch.qos.logback.core.Appender;
import ch.qos.logback.core.encoder.Encoder;
import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.datatype.jsr310.JavaTimeModule;
import lombok.Getter;
import org.correomqtt.business.fileprovider.ScriptingProvider;
import org.slf4j.LoggerFactory;
import org.slf4j.Marker;

import java.io.BufferedReader;
import java.io.File;
import java.io.IOException;
import java.io.InputStreamReader;
import java.io.PipedInputStream;
import java.io.PipedOutputStream;
import java.nio.charset.StandardCharsets;
import java.util.concurrent.CompletableFuture;

import static org.correomqtt.business.utils.LoggerUtils.SCRIPT_APPENDER_NAME;
import static org.correomqtt.business.utils.LoggerUtils.SCRIPT_COLOR_PATTERN_APPENDER_NAME;
import static org.correomqtt.business.utils.LoggerUtils.findPatternEncoder;
import static org.correomqtt.business.utils.LoggerUtils.findScriptAppender;

public class ScriptLoggerContext implements AutoCloseable {

    private static final org.slf4j.Logger LOGGER = LoggerFactory.getLogger(ScriptLoggerContext.class);
    private static final int MAX_WAIT = 5000;
    private static final int WAIT_STEP = 100;
    @Getter
    private final Logger scriptLogger;
    private CompletableFuture<Void> outputStreamListener;
    private PipedInputStream snk;
    private final Marker marker;
    private final Appender<ILoggingEvent> scriptAppender;
    private final ScriptFileAppender fileAppender;

    public ScriptLoggerContext(ExecutionDTO dto, Marker marker) {
        this.marker = marker;
        LoggerContext context = (LoggerContext) LoggerFactory.getILoggerFactory();
        scriptLogger = context.getLogger(dto.getScriptFile().getName() + "-" + dto.getExecutionId());

        Encoder<ILoggingEvent> encoder = findPatternEncoder(SCRIPT_COLOR_PATTERN_APPENDER_NAME);

        fileAppender = new ScriptFileAppender();
        fileAppender.setFile(ScriptingProvider.getInstance().getSingleScriptLogPath(dto.getScriptFile().getName(), dto.getExecutionId()));
        fileAppender.setEncoder(encoder);
        fileAppender.setContext(context);
        fileAppender.start();

        scriptAppender = findScriptAppender(SCRIPT_APPENDER_NAME);
        if (scriptAppender == null) {
            LOGGER.warn("No {} appender is configured for logback. No script output will be logged to standard log in gui, console or file.", SCRIPT_APPENDER_NAME);
        } else {
            scriptLogger.addAppender(scriptAppender);
        }
        scriptLogger.addAppender(fileAppender);
        scriptLogger.setAdditive(false);
    }

    public void connectSnk(PipedOutputStream out) throws IOException {
        snk = new PipedInputStream(out);
        outputStreamListener = CompletableFuture.runAsync(this::readLog);
    }


    private void readLog() {

        final int BUFFER_SIZE = 8192;
        try (
                final InputStreamReader isr = new InputStreamReader(snk, StandardCharsets.UTF_8);
                final BufferedReader br = new BufferedReader(isr, BUFFER_SIZE);
        ) {
            // stream log output
            String line;
            while ((line = br.readLine()) != null) {
                scriptLogger.info(marker, line);
            }
        } catch (IOException e) {
            // this is normal if SNK is closed.
            LOGGER.trace("Pipe to script log broke.", e);
        }
    }

    @Override
    public void close() throws Exception {
        scriptLogger.detachAppender(scriptAppender);
        scriptLogger.detachAppender(fileAppender);
        if (snk != null) {
            for (int currentWait = 0; currentWait < MAX_WAIT; currentWait += WAIT_STEP) {
                if (snk.available() <= 0) {
                    break;
                }
                Thread.sleep(WAIT_STEP);
            }
            if (LOGGER.isDebugEnabled() && snk.available() > 0)
                LOGGER.debug("SNK still has data available after {}ms => Stopping now.", MAX_WAIT);
            snk.close();
        }
        if (outputStreamListener != null) {
            outputStreamListener.cancel(true);
        }
        fileAppender.stop();
    }
}
