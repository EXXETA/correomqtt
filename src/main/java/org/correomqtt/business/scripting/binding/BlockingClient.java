package org.correomqtt.business.scripting.binding;

import org.graalvm.polyglot.HostAccess.Export;

import java.util.concurrent.LinkedBlockingDeque;
import java.util.concurrent.atomic.AtomicReference;
import java.util.function.BiConsumer;

public class BlockingClient {

    private final Client client;

    BlockingClient(Client client) {
        this.client = client;
    }

    @Export
    public void connect() throws Throwable {
        blockify((q, t) -> client.connect(
                () -> q.add(true),
                ex -> {
                    t.set(ex);
                    q.add(false);
                }));
    }

    @Export
    public void disconnect() throws Throwable {
        blockify((q, t) -> client.disconnect(
                () -> q.add(true),
                ex -> {
                    t.set(ex);
                    q.add(false);
                }));
    }

    private void blockify(BiConsumer<LinkedBlockingDeque<Boolean>, AtomicReference<Throwable>> callback) throws Throwable {
        LinkedBlockingDeque<Boolean> q = new LinkedBlockingDeque<>();
        AtomicReference<Throwable> t = new AtomicReference<>();
        callback.accept(q, t);
        if (Boolean.FALSE.equals(q.take())) {
            throw t.get();
        }
    }
}
