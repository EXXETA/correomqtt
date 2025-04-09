package org.correomqtt.core.scripting.binding;

import lombok.Getter;
import org.correomqtt.core.connection.ConnectionLifecycleTaskFactories;
import org.correomqtt.core.model.MessageDTO;
import org.correomqtt.core.model.MessageType;
import org.correomqtt.core.model.Qos;
import org.correomqtt.core.model.SubscriptionDTO;
import org.correomqtt.core.pubsub.IncomingMessageEvent;
import org.correomqtt.core.pubsub.PubSubTaskFactories;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import org.correomqtt.di.Observes;
import org.graalvm.polyglot.Context;
import org.graalvm.polyglot.HostAccess.Export;
import org.slf4j.Logger;
import org.slf4j.Marker;

import java.time.LocalDateTime;
import java.util.HashMap;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.function.Consumer;

import static org.correomqtt.core.connection.ConnectionState.DISCONNECTED_UNGRACEFUL;
import static org.correomqtt.core.connection.ConnectionState.RECONNECTING;
import static org.correomqtt.core.scripting.JsContextBuilder.CORREO_CONNECTION_ID;
import static org.correomqtt.core.scripting.JsContextBuilder.CORREO_SCRIPT_LOGGER;
import static org.correomqtt.core.scripting.JsContextBuilder.CORREO_SCRIPT_MARKER;
import static org.correomqtt.core.scripting.JsContextBuilder.CORREO_SCRIPT_QUEUE;

@DefaultBean
public class ClientImpl {

    private final ConnectionLifecycleTaskFactories connectionLifecycleTaskFactories;
    private final PubSubTaskFactories pubSubTaskFactories;
    private String connectionId;
    private Logger scriptLogger;
    private Queue queue;
    private Marker marker;

    @Getter
    private Context context;
    private AsyncClient asyncClient;
    private PromiseClient promiseClient;
    private BlockingClient blockingClient;
    private final Map<String, Consumer<String>> subscriptions = new HashMap<>();

    @Inject
    public ClientImpl(ConnectionLifecycleTaskFactories connectionLifecycleTaskFactories,
               PubSubTaskFactories pubSubTaskFactories) {
        this.connectionLifecycleTaskFactories = connectionLifecycleTaskFactories;
        this.pubSubTaskFactories = pubSubTaskFactories;
    }

    public void setContext(Context context) {
        this.context = context;
        connectionId = context.getPolyglotBindings().getMember(CORREO_CONNECTION_ID).as(String.class);
        marker = context.getPolyglotBindings().getMember(CORREO_SCRIPT_MARKER).as(Marker.class);
        scriptLogger = context.getPolyglotBindings().getMember(CORREO_SCRIPT_LOGGER).as(Logger.class);
        queue = context.getPolyglotBindings().getMember(CORREO_SCRIPT_QUEUE).as(Queue.class);
    }

    @Export
    public AsyncClient toAsync() {
        if (asyncClient == null) {
            asyncClient = new AsyncClient(this);
        }
        return asyncClient;
    }

    @Export
    public BlockingClient toBlocking() {
        if (blockingClient == null) {
            blockingClient = new BlockingClient(this);
        }
        return blockingClient;
    }

    @Export
    public PromiseClient toPromise() {
        if (promiseClient == null) {
            promiseClient = new PromiseClient(this);
        }
        return promiseClient;
    }

    void connect(Runnable onSuccess, Consumer<Throwable> onError) {
        connectionLifecycleTaskFactories.getConnectFactory().create(connectionId).onSuccess(() -> {
            scriptLogger.info(marker, "Client successful connected.");
            onSuccess.run();
        }).onProgress(ev -> {
            String msg = "Connection state changed to {}. Retry {}/{}";
            if (ev.getState() == RECONNECTING) {
                scriptLogger.warn(marker, msg, ev.getState(), ev.getRetries(), ev.getMaxRetries());
            } else if (ev.getState() == DISCONNECTED_UNGRACEFUL) {
                scriptLogger.error(marker, msg, ev.getState(), ev.getRetries(), ev.getMaxRetries());
            } else {
                scriptLogger.info(marker, msg, ev.getState(), ev.getRetries(), ev.getMaxRetries());
            }
        }).onError(r -> {
            scriptLogger.error(marker, "Client could not connect: {}", r.getUnexpectedError().getMessage());
            onError.accept(r.getUnexpectedError());
        }).run();
    }

    void disconnect(Runnable onSuccess, Consumer<Throwable> onError) {
        connectionLifecycleTaskFactories.getDisconnectFactory().create(connectionId).onSuccess(() -> {
            scriptLogger.info(marker, "Client successful disconnected.");
            onSuccess.run();
        }).onError(r -> {
            scriptLogger.error(marker, "Client could not disconnect: {}", r.getUnexpectedError().getMessage());
            onError.accept(r.getUnexpectedError());
        }).run();
    }

    void publish(String topic, int qos, boolean retained, String payload, Runnable onSuccess, Consumer<Throwable> onError) {

        pubSubTaskFactories.getPublishFactory().create(connectionId, MessageDTO.builder()
                .topic(topic)
                .qos(Qos.fromJsonValue(qos))
                .payload(payload)
                .isRetained(retained)
                .messageId(UUID.randomUUID().toString())
                .messageType(MessageType.OUTGOING)
                .dateTime(LocalDateTime.now())
                .build()
        ).onSuccess(() -> {
            scriptLogger.info(marker, "Published message to {} with qos {}.", topic, qos);
            onSuccess.run();
        }).onError(r -> {
            scriptLogger.error(marker, "Failed to publish message to {} with qos {}: {}", topic, qos, r.getUnexpectedError().getMessage());
            onError.accept(r.getUnexpectedError());
        }).run();
    }

    void subscribe(String topic, Integer qos, Runnable onSuccess, Consumer<Throwable> onError, Consumer<String> onIncomingMessage) {
        subscriptions.put(topic, onIncomingMessage);
        pubSubTaskFactories.getSubscribeFactory().create(connectionId, SubscriptionDTO.builder().topic(topic).qos(Qos.fromJsonValue(qos)).build()).onSuccess(() -> {
            scriptLogger.info(marker, "Subscribed to {} with qos {}.", topic, qos);
            onSuccess.run();
        }).onError(r -> {
            scriptLogger.error(marker, "Failed to subscribe to {} with qos {}: {}", topic, qos, r.getUnexpectedError().getMessage());
            onError.accept(r.getUnexpectedError());
        }).run();
    }

    void unsubscribe(String topic, Runnable onSuccess, Consumer<Throwable> onError) {
        pubSubTaskFactories.getUnsubscribeFactory().create(connectionId, SubscriptionDTO.builder().topic(topic).build()).onSuccess(() -> {
            scriptLogger.info(marker, "Unsubscribed from {}.", topic);
            onSuccess.run();
        }).onError(r -> {
            scriptLogger.error(marker, "Failed to unsubscribe from {}: {}", topic, r.getUnexpectedError().getMessage());
            onError.accept(r.getUnexpectedError());
        }).run();
    }

    void unsubscribeAll(Runnable onSuccess, Consumer<Throwable> onError) {

        AtomicBoolean success = new AtomicBoolean(true);
        AtomicInteger count = new AtomicInteger(subscriptions.size());

        for (String topic : subscriptions.keySet()) {
            unsubscribe(topic, () -> {
                int c = count.decrementAndGet();
                if (c == 0 && success.get()) {
                    onSuccess.run();
                } else if (c == 0) {
                    onError.accept(null);
                }
            }, r -> {
                int c = count.decrementAndGet();
                if (c == 0 && success.get()) {
                    onSuccess.run();
                } else if (c == 0) {
                    onError.accept(null);
                }
                success.set(false);
            });
        }
    }

    @SuppressWarnings("unused")
    public void onSubscribe(@Observes IncomingMessageEvent event) {
        queue.add(new QueueEvent(() -> {
            String topic = event.getSubscriptionDTO().getTopic();
            if (subscriptions.containsKey(topic)) {
                Consumer<String> sub = subscriptions.get(topic);
                sub.accept(event.getMessageDTO().getPayload());
            }
        }));
    }
}
