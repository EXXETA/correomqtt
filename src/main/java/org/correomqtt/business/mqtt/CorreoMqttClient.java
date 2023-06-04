package org.correomqtt.business.mqtt;

import org.correomqtt.business.model.MessageDTO;
import org.correomqtt.business.model.SubscriptionDTO;

import javax.net.ssl.SSLException;
import java.util.Set;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeoutException;
import java.util.function.Consumer;

public interface CorreoMqttClient {

    void connect() throws InterruptedException, ExecutionException, TimeoutException, SSLException;

    void disconnect(boolean graceful);

    void publish(MessageDTO messageDTO) throws InterruptedException, ExecutionException, TimeoutException;

    void subscribe(SubscriptionDTO subscriptionDTO, Consumer<MessageDTO> incomingCallback) throws InterruptedException, ExecutionException, TimeoutException;

    void unsubscribe(SubscriptionDTO subscriptionDTO);

    Set<SubscriptionDTO> getSubscriptions();
}
