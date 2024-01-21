package org.correomqtt.core.scripting.binding;

import org.graalvm.polyglot.Context;
import org.graalvm.polyglot.HostAccess.Export;
import org.graalvm.polyglot.Value;

import java.util.concurrent.LinkedBlockingDeque;
import java.util.concurrent.atomic.AtomicReference;
import java.util.function.BiConsumer;
import java.util.function.Consumer;

import static org.correomqtt.core.scripting.JsContextBuilder.CORREO_ASYNC_LATCH;

public class PromiseClient {

    private final Client client;

    private final AsyncLatch asyncLatch;

    PromiseClient(Client client) {
        this.client = client;
        Context context = client.getContext();
        this.asyncLatch = context.getPolyglotBindings().getMember(CORREO_ASYNC_LATCH).as(AsyncLatch.class);
    }


    @Export
    public BlockingClient toBlocking() {
        return client.toBlocking();
    }

    @Export
    public AsyncClient toAsync() {
        return client.toAsync();
    }

    @Export
    public PromiseInterface connect() {
        return (resolve, reject) -> asyncify((q, t) -> client.connect(() -> q.add(true), ex -> {
            t.set(ex);
            q.add(false);
        }), resolve, reject);
    }

    @Export
    public PromiseInterface disconnect() {
        return (resolve, reject) -> asyncify((q, t) -> client.disconnect(() -> q.add(true), ex -> {
            t.set(ex);
            q.add(false);
        }), resolve, reject);
    }

    @Export
    public PromiseInterface publish(String topic, int qos) {
        return (resolve, reject) -> asyncify((q, t) -> client.publish(topic, qos, false, "", () -> q.add(true), ex -> {
            t.set(ex);
            q.add(false);
        }), resolve, reject);
    }

    @Export
    public PromiseInterface publish(String topic, int qos, String payload) {
        return (resolve, reject) -> asyncify((q, t) -> client.publish(topic, qos, false, payload, () -> q.add(true), ex -> {
            t.set(ex);
            q.add(false);
        }), resolve, reject);
    }

    @Export
    public PromiseInterface publish(String topic, int qos, boolean retained) {
        return (resolve, reject) -> asyncify((q, t) -> client.publish(topic, qos, retained, "", () -> q.add(true), ex -> {
            t.set(ex);
            q.add(false);
        }), resolve, reject);
    }

    @Export
    public PromiseInterface publish(String topic, int qos, boolean retained, String payload) {
        return (resolve, reject) -> asyncify((q, t) -> client.publish(topic, qos, retained, payload, () -> q.add(true), ex -> {
            t.set(ex);
            q.add(false);
        }), resolve, reject);
    }

    @Export
    public PromiseInterface subscribe(String topic, Integer qos, Consumer<String> onIncomingMessage) {
        return (resolve, reject) -> asyncify((q, t) -> client.subscribe(topic, qos, () -> q.add(true), ex -> {
            t.set(ex);
            q.add(false);
        }, onIncomingMessage), resolve, reject);

    }

    @Export
    public PromiseInterface unsubscribe(String topic) {
        return (resolve, reject) -> asyncify((q, t) -> client.unsubscribe(topic, () -> q.add(true), ex -> {
            t.set(ex);
            q.add(false);
        }), resolve, reject);

    }

    @Export
    public PromiseInterface unsubscribeAll() {
        return (resolve, reject) -> asyncify((q, t) -> client.unsubscribeAll(() -> q.add(true), ex -> {
            t.set(ex);
            q.add(false);
        }), resolve, reject);
    }

    private void asyncify(BiConsumer<LinkedBlockingDeque<Boolean>, AtomicReference<Throwable>> callback, Value onSuccess, Value onError) throws InterruptedException {
        asyncLatch.increase();
        LinkedBlockingDeque<Boolean> q = new LinkedBlockingDeque<>();
        AtomicReference<Throwable> t = new AtomicReference<>();
        callback.accept(q, t);
        if (Boolean.TRUE.equals(q.take())) {
            onSuccess.executeVoid();
        } else {
            onError.executeVoid();
        }
        asyncLatch.decrease();
    }

}
