package org.correomqtt.core.scripting.binding;

import org.graalvm.polyglot.Context;
import org.graalvm.polyglot.HostAccess.Export;

import java.util.concurrent.LinkedBlockingDeque;
import java.util.concurrent.atomic.AtomicReference;
import java.util.function.BiConsumer;
import java.util.function.Consumer;

import static org.correomqtt.core.scripting.JsContextBuilder.CORREO_ASYNC_LATCH;

public class AsyncClient {

    private final ClientImpl client;

    private final AsyncLatch asyncLatch;

    AsyncClient(ClientImpl client) {
        this.client = client;
        Context context = client.getContext();
        this.asyncLatch = context.getPolyglotBindings().getMember(CORREO_ASYNC_LATCH).as(AsyncLatch.class);
    }

    @Export
    public BlockingClient toBlocking() {
        return client.toBlocking();
    }

    @Export
    public PromiseClient toPromised() {
        return client.toPromise();
    }

    @Export
    public void connect() throws InterruptedException {
        connect(() -> {}, () -> {});
    }

    @Export
    public void connect(Runnable onSuccess) throws InterruptedException {
        connect(onSuccess, () -> {});
    }

    @Export
    public void connect(Runnable onSuccess, Runnable onError) throws InterruptedException {
        asyncify((q, t) -> client.connect(() -> q.add(true), ex -> {
            t.set(ex);
            q.add(false);
        }), onSuccess, onError);
    }

    @Export
    public void disconnect() throws InterruptedException {
        disconnect(() -> {}, () -> {});
    }

    @Export
    public void disconnect(Runnable onSuccess) throws InterruptedException {
        disconnect(onSuccess, () -> {});
    }

    @Export
    public void disconnect(Runnable onSuccess, Runnable onError) throws InterruptedException {
        asyncify((q, t) -> client.disconnect(() -> q.add(true), ex -> {
            t.set(ex);
            q.add(false);
        }), onSuccess, onError);
    }

    @Export
    public void publish(String topic, int qos) throws InterruptedException {
        publish(topic, qos, false, "", () -> {}, () -> {});
    }

    @Export
    public void publish(String topic, int qos, boolean retained) throws InterruptedException {
        publish(topic, qos, retained, "", () -> {}, () -> {});
    }

    @Export
    public void publish(String topic, int qos, Runnable onSuccess) throws InterruptedException {
        publish(topic, qos, false, "", onSuccess, () -> {});
    }

    @Export
    public void publish(String topic, int qos, boolean retained, Runnable onSuccess) throws InterruptedException {
        publish(topic, qos, retained, "", onSuccess, () -> {});
    }


    @Export
    public void publish(String topic, int qos, String payload) throws InterruptedException {
        publish(topic, qos, false, payload, () -> {}, () -> {});
    }

    @Export
    public void publish(String topic, int qos, boolean retained, String payload) throws InterruptedException {
        publish(topic, qos, retained, payload, () -> {}, () -> {});
    }

    @Export
    public void publish(String topic, int qos, String payload, Runnable onSuccess) throws InterruptedException {
        publish(topic, qos, false, payload, onSuccess, () -> {});
    }

    @Export
    public void publish(String topic, int qos, boolean retained, String payload, Runnable onSuccess) throws InterruptedException {
        publish(topic, qos, retained, payload, onSuccess, () -> {});
    }

    @Export
    public void publish(String topic, int qos, String payload, Runnable onSuccess, Runnable onError) throws InterruptedException {
        publish(topic, qos, false, payload, onSuccess, onError);
    }

    @Export
    public void publish(String topic, int qos, boolean retained, Runnable onSuccess, Runnable onError) throws InterruptedException {
        asyncify((q, t) -> client.publish(topic, qos, retained, "", () -> q.add(true), ex -> {
            t.set(ex);
            q.add(false);
        }), onSuccess, onError);
    }

    @Export
    public void publish(String topic, Integer qos, boolean retained, String payload, Runnable onSuccess, Runnable onError) throws InterruptedException {
        asyncify((q, t) -> client.publish(topic, qos, retained, payload, () -> q.add(true), ex -> {
            t.set(ex);
            q.add(false);
        }), onSuccess, onError);
    }

    @Export
    public void subscribe(String topic, Integer qos, Consumer<String> onIncomingMessage) throws InterruptedException {
        subscribe(topic, qos, () -> {}, () -> {}, onIncomingMessage);
    }

    @Export
    public void subscribe(String topic, Integer qos, Runnable onSuccess, Consumer<String> onIncomingMessage) throws InterruptedException {
        subscribe(topic, qos, onSuccess, () -> {}, onIncomingMessage);
    }

    @Export
    public void subscribe(String topic, Integer qos, Runnable onSuccess, Runnable onError, Consumer<String> onIncomingMessage) throws InterruptedException {
        asyncify((q, t) -> client.subscribe(topic, qos, () -> q.add(true), ex -> {
            t.set(ex);
            q.add(false);
        }, onIncomingMessage), onSuccess, onError);
    }

    @Export
    public void unsubscribe(String topic) throws InterruptedException {
        unsubscribe(topic, () -> {}, () -> {});
    }

    @Export
    public void unsubscribe(String topic, Runnable onSuccess) throws InterruptedException {
        unsubscribe(topic, onSuccess, () -> {});
    }

    @Export
    public void unsubscribe(String topic, Runnable onSuccess, Runnable onError) throws InterruptedException {
        asyncify((q, t) -> client.unsubscribe(topic, () -> q.add(true), ex -> {
            t.set(ex);
            q.add(false);
        }), onSuccess, onError);
    }

    @Export
    public void unsubscribeAll() throws InterruptedException {
        unsubscribeAll(() -> {}, () -> {});
    }

    @Export
    public void unsubscribeAll(Runnable onSuccess) throws InterruptedException {
        unsubscribeAll(onSuccess, () -> {});
    }

    @Export
    public void unsubscribeAll(Runnable onSuccess, Runnable onError) throws InterruptedException {
        asyncify((q, t) -> client.unsubscribeAll(() -> q.add(true), ex -> {
            t.set(ex);
            q.add(false);
        }), onSuccess, onError);

    }

    private void asyncify(BiConsumer<LinkedBlockingDeque<Boolean>, AtomicReference<Throwable>> callback, Runnable onSuccess, Runnable onError) throws InterruptedException {
        asyncLatch.increase();
        LinkedBlockingDeque<Boolean> q = new LinkedBlockingDeque<>();
        AtomicReference<Throwable> t = new AtomicReference<>();
        callback.accept(q, t);
        if (Boolean.TRUE.equals(q.take())) {
            onSuccess.run();
        } else {
            onError.run();
        }
        asyncLatch.decrease();
    }

}
