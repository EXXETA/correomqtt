package org.correomqtt.core.utils;

import ch.qos.logback.classic.Logger;
import ch.qos.logback.classic.LoggerContext;
import ch.qos.logback.classic.spi.ILoggingEvent;
import ch.qos.logback.core.Appender;
import ch.qos.logback.core.FileAppender;
import ch.qos.logback.core.encoder.Encoder;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.core.model.ConnectionConfigDTO;
import org.slf4j.LoggerFactory;
import org.slf4j.Marker;
import org.slf4j.MarkerFactory;

import org.correomqtt.di.Inject;
import java.util.Iterator;

@DefaultBean
public class LoggerUtils {

    public static final String SCRIPT_APPENDER_NAME = "SCRIPT";

    public static final String SCRIPT_COLOR_PATTERN_APPENDER_NAME = "SCRIPT_DUMMY_COLOR_PATTERN";

    public static final String SCRIPT_PATTERN_APPENDER_NAME = "SCRIPT_DUMMY_PATTERN";
    private final ConnectionManager connectionManager;

    @Inject
    public LoggerUtils(ConnectionManager connectionManager){
        // private constructor
        this.connectionManager = connectionManager;
    }

    public Marker getConnectionMarker(String connectionId) {
        ConnectionConfigDTO connectionConfig = connectionManager.getConfig(connectionId);
        if (connectionConfig == null) {
            return MarkerFactory.getMarker("Unknown");
        }
        return MarkerFactory.getMarker(connectionConfig.getName());
    }

    public static Encoder<ILoggingEvent> findPatternEncoder(String appenderName) {
        LoggerContext context = (LoggerContext) LoggerFactory.getILoggerFactory();
        for (Logger candidate : context.getLoggerList()) {
            for (Iterator<Appender<ILoggingEvent>> index = candidate.iteratorForAppenders(); index.hasNext(); ) {
                Appender<ILoggingEvent> appender = index.next();
                if (appenderName.equals(appender.getName()) && appender instanceof FileAppender<ILoggingEvent> fileappender) {
                    return fileappender.getEncoder();
                }
            }
        }
        return null;
    }

    public static Appender<ILoggingEvent> findLogAppender(String appenderName) {
        LoggerContext context = (LoggerContext) LoggerFactory.getILoggerFactory();
        for (Logger candidate : context.getLoggerList()) {
            for (Iterator<Appender<ILoggingEvent>> index = candidate.iteratorForAppenders(); index.hasNext(); ) {
                Appender<ILoggingEvent> appender = index.next();
                if (appender.getName().equals(appenderName)) {
                    return appender;
                }
            }
        }
        return null;
    }
}
