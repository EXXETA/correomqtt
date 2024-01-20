package org.correomqtt.core.log;

import ch.qos.logback.classic.encoder.PatternLayoutEncoder;
import ch.qos.logback.classic.spi.ILoggingEvent;
import ch.qos.logback.core.AppenderBase;
import lombok.Getter;
import lombok.Setter;
import org.correomqtt.core.eventbus.EventBus;
import org.correomqtt.core.eventbus.Subscribe;

import java.nio.charset.StandardCharsets;
import java.util.List;
import java.util.Set;
import java.util.concurrent.CopyOnWriteArrayList;
import java.util.stream.Collectors;

@Setter
@Getter
public class LogDispatchAppender extends AppenderBase<ILoggingEvent> {

    private PatternLayoutEncoder encoder;

    private final List<String> cache = new CopyOnWriteArrayList<>();

    @Override
    public void start() {
        if (this.encoder == null) {
            addError("No encoder set for the appender named [" + name + "].");
            return;
        }
        encoder.start();
        super.start();
        EventBus.register(this);
    }

    @SuppressWarnings("unused")
    @Subscribe(PopLogCache.class)
    public synchronized void popCache() {
        Set<String> sentCache = cache.stream()
                .filter(msg -> EventBus.fire(new LogEvent(msg)) > 0)
                .collect(Collectors.toSet());

        cache.removeAll(sentCache);
    }

    @Override
    public void stop() {
        EventBus.unregister(this);
        super.stop();
    }

    @Override
    protected void append(ILoggingEvent eventObject) {
        String logMsg = new String(this.encoder.encode(eventObject), StandardCharsets.UTF_8);
        if (EventBus.fire(new LogEvent(logMsg)) == 0) {
            this.cache.add(logMsg);
        }
    }
}