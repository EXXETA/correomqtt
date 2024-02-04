package org.correomqtt.core.scripting.binding;

import org.graalvm.polyglot.HostAccess.Export;
import org.slf4j.Logger;

import java.util.concurrent.LinkedBlockingDeque;

public class Queue {

    private final LinkedBlockingDeque<QueueEvent> events = new LinkedBlockingDeque<>();
    private final Logger scriptLogger;

    public Queue(Logger scriptLogger) {
        this.scriptLogger = scriptLogger;
    }

    @Export
    public void add(QueueEvent event) {
        events.add(event);
    }

    @Export
    public void process() throws InterruptedException {
        QueueEvent event;
        do {
            event = events.take();
            if (event.isContinue())
                continue;

            event.callback().run();

        } while (!event.isContinue());
    }

    @SuppressWarnings("unused")
    @Export
    public void jumpOut() {
        if (!events.offer(new QueueEvent(null))) {
            scriptLogger.error("Internal event queue is out of capacity.");
        }
    }

}
