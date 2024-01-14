package org.correomqtt.business.scripting.binding;

import org.graalvm.polyglot.HostAccess.Export;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.concurrent.LinkedBlockingDeque;

public class AsyncLatch {

    private static final Logger LOGGER = LoggerFactory.getLogger(AsyncLatch.class);
    private int count = 0;
    private final LinkedBlockingDeque<Integer> countQueue = new LinkedBlockingDeque<>();

    @Export
    public synchronized void increase() {
        count++;
    }

    @Export

    public synchronized void decrease() {
        if (!countQueue.offer(1)) {
            LOGGER.error("Unable to decrease AsyncLatch,");
        }
    }

    @Export
    public synchronized void join() throws InterruptedException {
        while (count > 0) {
            LOGGER.info("Waiting for asynchronous callbacks return: {}", count);
            count -= countQueue.take();
        }
    }
}
