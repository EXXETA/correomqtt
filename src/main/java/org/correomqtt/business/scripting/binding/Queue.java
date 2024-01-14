package org.correomqtt.business.scripting.binding;

import org.graalvm.polyglot.HostAccess.Export;

import java.util.concurrent.LinkedBlockingDeque;

public class Queue {

    private final LinkedBlockingDeque<QueueEvent> events = new LinkedBlockingDeque<>();

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

    @Export
    public void jumpOut() {
        if (!events.offer(new QueueEvent(null))) {
            // TODO scriptLogger.error("Internal event queue is out of capacity.");
        }
    }


}
