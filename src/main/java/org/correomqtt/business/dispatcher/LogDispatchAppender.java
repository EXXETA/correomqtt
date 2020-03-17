package com.exxeta.correomqtt.business.dispatcher;

import ch.qos.logback.classic.encoder.PatternLayoutEncoder;
import ch.qos.logback.classic.spi.ILoggingEvent;
import ch.qos.logback.core.AppenderBase;

import java.nio.charset.StandardCharsets;

public class LogDispatchAppender extends AppenderBase<ILoggingEvent> {

    private PatternLayoutEncoder encoder;

    @Override
    public void start() {
        if (this.encoder == null) {
            addError("No encoder set for the appender named [" + name + "].");
            return;
        }

        encoder.start();

        super.start();
    }


    @Override
    protected void append(ILoggingEvent eventObject) {
        String logMsg = new String(this.encoder.encode(eventObject), StandardCharsets.UTF_8);
        LogDispatcher.getInstance().log(logMsg);

    }

    public PatternLayoutEncoder getEncoder() {
        return encoder;
    }

    public void setEncoder(PatternLayoutEncoder encoder) {
        this.encoder = encoder;
    }
}
