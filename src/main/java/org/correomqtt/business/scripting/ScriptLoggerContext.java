package org.correomqtt.business.scripting;

import ch.qos.logback.classic.Logger;
import ch.qos.logback.classic.LoggerContext;
import ch.qos.logback.classic.encoder.PatternLayoutEncoder;
import ch.qos.logback.classic.spi.ILoggingEvent;
import ch.qos.logback.core.Appender;
import ch.qos.logback.core.OutputStreamAppender;
import org.slf4j.LoggerFactory;

import java.io.PipedOutputStream;
import java.util.Iterator;

public class ScriptLoggerContext implements AutoCloseable {

    private static final org.slf4j.Logger LOGGER = LoggerFactory.getLogger(ScriptLoggerContext.class);
    private final PatternLayoutEncoder encoder;
    private final OutputStreamAppender<ILoggingEvent> outAppender;
    private final Logger scriptLogger;

    public ScriptLoggerContext(PipedOutputStream out, String executionId, String filename) {

        LoggerContext context = (LoggerContext) LoggerFactory.getILoggerFactory();

        encoder = new PatternLayoutEncoder();
        encoder.setContext(context);
        encoder.setPattern("%cyan(%date{'HH:mm:ss,SSS'});%highlight(%5.5level);%yellow(%logger{%20.20});%highlight(%0.0level%msg%xThrowable{full}%n)");
        encoder.start();

        outAppender = new OutputStreamAppender<>();
        outAppender.setName(filename + " " + executionId);
        outAppender.setContext(context);
        outAppender.setEncoder(encoder);
        outAppender.setOutputStream(out);

        Appender<ILoggingEvent> scriptAppender = null;
        for (Logger candidate : context.getLoggerList()) {
            for (Iterator<Appender<ILoggingEvent>> index = candidate.iteratorForAppenders(); index.hasNext(); ) {
                Appender<ILoggingEvent> appender = index.next();
                if (appender.getName().equals("SCRIPT")) {
                    scriptAppender = appender;
                }
            }
        }

        outAppender.start();

        scriptLogger = context.getLogger(filename + " " + executionId);
        scriptLogger.addAppender(outAppender);
        if (scriptAppender == null) {
            LOGGER.warn("No SCRIPT appender is configured for logback. No script output will be logged to standard log in gui, console or file.");
        } else {
            scriptLogger.addAppender(scriptAppender);
        }
        scriptLogger.setAdditive(false);
    }


    public org.slf4j.Logger getScriptLogger() {
        return scriptLogger;
    }

    @Override
    public void close() throws Exception {
        outAppender.stop();
        encoder.stop();
    }
}
