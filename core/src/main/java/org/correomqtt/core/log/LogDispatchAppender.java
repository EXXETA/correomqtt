package org.correomqtt.core.log;

import ch.qos.logback.classic.encoder.PatternLayoutEncoder;
import ch.qos.logback.classic.spi.ILoggingEvent;
import ch.qos.logback.core.AppenderBase;
import lombok.Getter;
import lombok.Setter;
import org.correomqtt.core.eventbus.EventBus;

import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.CopyOnWriteArrayList;

@Setter
@Getter
public class LogDispatchAppender extends AppenderBase<ILoggingEvent> {

    private PatternLayoutEncoder encoder;

    private final List<String> cache = new CopyOnWriteArrayList<>();

    private EventBus eventBus;

    @Override
    public void start() {
        if (this.encoder == null) {
            addError("No encoder set for the appender named [" + name + "].");
            return;
        }
        encoder.start();
        super.start();
    }

    public synchronized List<String> popCache(EventBus eventBus) {
        ArrayList<String> r = new ArrayList<>(cache);
        cache.clear();
        this.eventBus = eventBus;
        return r;
    }

    @Override
    protected void append(ILoggingEvent eventObject) {
        String logMsg = new String(this.encoder.encode(eventObject), StandardCharsets.UTF_8);
        if (eventBus == null || eventBus.fireAsync(new LogEvent(logMsg)) == 0) {
            cache.add(logMsg);
        }
    }
}