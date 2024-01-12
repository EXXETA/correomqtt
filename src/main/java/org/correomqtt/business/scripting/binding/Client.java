package org.correomqtt.business.scripting.binding;

import org.correomqtt.business.connection.ConnectTask;
import org.correomqtt.business.connection.DisconnectTask;
import org.correomqtt.business.eventbus.EventBus;
import org.correomqtt.business.eventbus.Subscribe;
import org.correomqtt.business.model.MessageDTO;
import org.correomqtt.business.model.Qos;
import org.correomqtt.business.model.SubscriptionDTO;
import org.correomqtt.business.pubsub.IncomingMessageEvent;
import org.correomqtt.business.pubsub.PublishTask;
import org.correomqtt.business.pubsub.SubscribeTask;
import org.correomqtt.business.pubsub.UnsubscribeTask;
import org.graalvm.polyglot.Context;
import org.graalvm.polyglot.HostAccess.Export;
import org.graalvm.polyglot.Value;
import org.slf4j.Logger;

import java.util.HashMap;
import java.util.Map;
import java.util.concurrent.LinkedBlockingDeque;
import java.util.function.Consumer;

import static org.correomqtt.business.scripting.JsContextBuilder.CORREO_CONNECTION_ID;
import static org.correomqtt.business.scripting.JsContextBuilder.CORREO_SCRIPT_LOGGER;

public class Client {

    private final String connectionId;
    private final Logger scriptLogger;
    private AsyncClient asyncClient;
    private PromiseClient promiseClient;
    private BlockingClient blockingClient;

    public interface SubscriptionInterface {
        @SuppressWarnings("unused")
        Value handle(Value callback) throws InterruptedException;
    }

    public interface QueueInterface {
        @SuppressWarnings("unused")
        void work() throws InterruptedException;

        @SuppressWarnings("unused")
        void finish();
    }

    private final LinkedBlockingDeque<IncomingMessageEvent> eventQueue = new LinkedBlockingDeque<>();
    private final Map<String, Value> subscriptions = new HashMap<>();
    private final Context context;

    Client() {
        context = Context.getCurrent();
        connectionId = context.getPolyglotBindings().getMember(CORREO_CONNECTION_ID).as(String.class);
        scriptLogger = context.getPolyglotBindings().getMember(CORREO_SCRIPT_LOGGER).as(Logger.class);
        EventBus.register(this);
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
        new ConnectTask(connectionId)
                .onSuccess(() -> {
                    scriptLogger.info("Client successful connected.");
                    onSuccess.run();
                })
                .onError(r -> {
                    scriptLogger.error("Client could not connect: {}", r.getUnexpectedError().getMessage());
                    onError.accept(r.getUnexpectedError());
                })
                .run();
    }

    void disconnect(Runnable onSuccess, Consumer<Throwable> onError) {
        new DisconnectTask(connectionId)
                .onSuccess(() -> {
                    scriptLogger.info("Client successful disconnected.");
                    onSuccess.run();
                })
                .onError(r -> {
                    scriptLogger.error("Client could not disconnect: {}", r.getUnexpectedError().getMessage());
                    onError.accept(r.getUnexpectedError());
                })
                .run();
    }

    /**
     * @param topic   Topic
     * @param qos     QoS
     * @param payload Payload
     * @return Value true if success, other false
     */
    @SuppressWarnings("unused")
    @Export
    public Value publish(String topic, Integer qos, String payload) throws InterruptedException {

        LinkedBlockingDeque<Value> queue = new LinkedBlockingDeque<>();
        new PublishTask(connectionId, MessageDTO.builder()
                .topic(topic)
                .qos(Qos.fromJsonValue(qos))
                .payload(payload).build())
                .onSuccess(() -> {
                    scriptLogger.info("Published message to {} with qos {}.", topic, qos);
                    offerTrue(queue);
                })
                .onError(r -> {
                    scriptLogger.error("Failed to publish message to {} with qos {}: {}", topic, qos, r.getUnexpectedError().getMessage());
                    offerFalse(queue);
                })
                .run();

        return queue.take();
    }

    /**
     * @param topic Topic
     * @param qos   QoS
     * @return SubscriptionInterface for callback -> handle()
     */
    @SuppressWarnings("unused")
    @Export
    public Value subscribe(String topic, Integer qos) {
        return Value.asValue((SubscriptionInterface) callback -> {
            LinkedBlockingDeque<Value> queue = new LinkedBlockingDeque<>();
            subscriptions.put(topic, callback);
            new SubscribeTask(connectionId, SubscriptionDTO.builder()
                    .topic(topic)
                    .qos(Qos.fromJsonValue(qos))
                    .build())
                    .onSuccess(() -> {
                        scriptLogger.info("Subscribed to {} with qos {}.", topic, qos);
                        offerTrue(queue);
                    })
                    .onError(r -> {
                        scriptLogger.error("Failed to subscribe to {} with qos {}: {}", topic, qos, r.getUnexpectedError().getMessage());
                        offerFalse(queue);
                    })
                    .run();
            return queue.take();
        });
    }


    /**
     * @param topic Topic
     * @return Value true if success, other false
     */
    @SuppressWarnings("unused")
    @Export
    public Value unsubscribe(String topic) throws InterruptedException {
        LinkedBlockingDeque<Value> queue = new LinkedBlockingDeque<>();
        new UnsubscribeTask(connectionId, SubscriptionDTO.builder()
                .topic(topic)
                .build())
                .onSuccess(() -> {
                    scriptLogger.info("Unsubscribed from {}.", topic);
                    offerTrue(queue);
                })
                .onError(r -> {
                    scriptLogger.error("Failed to unsubscribe from {}: {}", topic, r.getUnexpectedError().getMessage());
                    offerFalse(queue);
                })
                .run();
        return queue.take();
    }

    /**
     * @return Value true if success, other false
     */
    @SuppressWarnings("unused")
    @Export
    public Value unsubscribeAll() throws InterruptedException {
        boolean success = true;
        for (String topic : subscriptions.keySet()) {
            success = unsubscribe(topic).asBoolean() && success;
        }
        return Value.asValue(success);
    }


    /**
     * @return Value true if success, other false
     */
    @SuppressWarnings("unused")
    @Export
    public Value queue() {
        return Value.asValue(new QueueInterface() {
            @Override
            public void work() throws InterruptedException {
                IncomingMessageEvent event;
                do {
                    event = eventQueue.take();
                    if (event.getConnectionId() == null)
                        continue;
                    String topic = event.getSubscriptionDTO().getTopic();
                    if (subscriptions.containsKey(topic)) {
                        Value sub = subscriptions.get(topic);
                        sub.execute(event.getMessageDTO().getPayload());
                    }

                } while (event.getConnectionId() != null);
            }

            @Override
            public void finish() {
                if (!eventQueue.offer(new IncomingMessageEvent(null, null, null))) {
                    scriptLogger.error("Internal event queue is out of capacity.");
                }
            }
        });
    }

    @SuppressWarnings("unused")
    public void onSubscribe(@Subscribe IncomingMessageEvent event) {
        eventQueue.add(event);
    }

    private void offerTrue(LinkedBlockingDeque<Value> queue) {
        offerToQueue(queue, Value.asValue(true));
    }

    private void offerFalse(LinkedBlockingDeque<Value> queue) {
        offerToQueue(queue, Value.asValue(false));
    }

    private void offerToQueue(LinkedBlockingDeque<Value> queue, Value value) {
        if (!queue.offer(value)) {
            scriptLogger.error("Internal event queue is out of capacity.");
        }
    }

}
