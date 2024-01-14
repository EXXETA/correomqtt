package org.correomqtt.business.scripting.binding;

import org.graalvm.polyglot.HostAccess.Export;
import org.graalvm.polyglot.Value;

import java.util.concurrent.LinkedBlockingDeque;
import java.util.concurrent.atomic.AtomicReference;
import java.util.function.BiConsumer;

public class BlockingClient {

    private final Client client;

    BlockingClient(Client client) {
        this.client = client;
    }


    @Export
    public PromiseClient toPromised() {
        return new ClientFactory().getPromiseClient();
    }

    @Export
    public AsyncClient toAsync() {
        return new ClientFactory().getAsyncClient();
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

    @Export
    public void publish(String topic, int qos) throws Throwable {
        publish(topic, qos, false, null);
    }

    @Export
    public void publish(String topic, int qos, boolean retained) throws Throwable {
        publish(topic, qos, retained, null);
    }

    @Export
    public void publish(String topic, int qos, String payload) throws Throwable {
        publish(topic, qos, false, payload);
    }

    @Export
    public void publish(String topic, int qos, boolean retained, String payload) throws Throwable {
        blockify((q, t) -> client.publish(topic, qos, retained, payload,
                () -> q.add(true),
                ex -> {
                    t.set(ex);
                    q.add(false);
                }));
    }

    @Export
    public void subscribe(String topic, Integer qos, Value onIncomingMessage) throws Throwable {
        blockify((q, t) -> client.subscribe(topic, qos,
                () -> q.add(true),
                ex -> {
                    t.set(ex);
                    q.add(false);
                },
                onIncomingMessage::executeVoid));
    }


    @Export
    public void unsubscribe(String topic) throws Throwable {
        blockify((q, t) -> client.unsubscribe(topic,
                () -> q.add(true),
                ex -> {
                    t.set(ex);
                    q.add(false);
                }));
    }

    @Export
    public void unsubscribeAll() throws Throwable {
        blockify((q, t) -> client.unsubscribeAll(
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
