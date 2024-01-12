package org.correomqtt.business.scripting;

import ch.qos.logback.classic.Logger;
import ch.qos.logback.classic.LoggerContext;
import ch.qos.logback.classic.encoder.PatternLayoutEncoder;
import ch.qos.logback.classic.spi.ILoggingEvent;
import ch.qos.logback.core.OutputStreamAppender;
import org.slf4j.LoggerFactory;

import java.io.PipedOutputStream;

public class ScriptLoggerContext implements AutoCloseable{
    private final LoggerContext context;
    private final PatternLayoutEncoder encoder;
    private final OutputStreamAppender<ILoggingEvent> appender;
    private final Logger logger;

    public ScriptLoggerContext(PipedOutputStream out, String filename) {

        context = (LoggerContext) LoggerFactory.getILoggerFactory();

        encoder = new PatternLayoutEncoder();
        encoder.setContext(context);
        encoder.setPattern("%cyan(%date{'HH:mm:ss,SSS'});%highlight(%5.5level);%yellow(%logger{20});%highlight(%0.0level%msg%xThrowable{full}%n)");
        encoder.start();

        appender = new OutputStreamAppender<>();
        appender.setName("OutputStream Appender");
        appender.setContext(context);
        appender.setEncoder(encoder);
        appender.setOutputStream(out);

        appender.start();

        logger = context.getLogger(filename);
        logger.addAppender(appender);
        logger.setAdditive(false);
    }

    public org.slf4j.Logger getLogger(){
        return logger;
    }

    @Override
    public void close() throws Exception {
        appender.stop();
        encoder.stop();
        context.stop();
    }
}
