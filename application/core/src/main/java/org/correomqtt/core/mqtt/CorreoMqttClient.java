package org.correomqtt.core.mqtt;

import org.correomqtt.core.connection.ConnectionState;
import org.correomqtt.core.model.MessageDTO;
import org.correomqtt.core.model.SubscriptionDTO;

import javax.net.ssl.SSLException;
import java.util.Set;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeoutException;
import java.util.function.Consumer;

public interface CorreoMqttClient {

    void connect() throws InterruptedException, ExecutionException, TimeoutException, SSLException;

    void disconnect();

    void publish(MessageDTO messageDTO) throws InterruptedException, ExecutionException, TimeoutException;

    void subscribe(SubscriptionDTO subscriptionDTO, Consumer<MessageDTO> incomingCallback) throws InterruptedException, ExecutionException, TimeoutException;

    void unsubscribe(SubscriptionDTO subscriptionDTO);

    Set<SubscriptionDTO> getSubscriptions();

    ConnectionState getState();

}
