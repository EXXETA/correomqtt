package org.correomqtt.business.mqtt;

import com.hivemq.client.mqtt.MqttClient;
import com.hivemq.client.mqtt.lifecycle.MqttClientDisconnectedContext;
import com.hivemq.client.mqtt.mqtt5.Mqtt5AsyncClient;
import com.hivemq.client.mqtt.mqtt5.Mqtt5BlockingClient;
import com.hivemq.client.mqtt.mqtt5.Mqtt5ClientBuilder;
import com.hivemq.client.mqtt.mqtt5.message.connect.Mqtt5ConnectBuilder;
import com.hivemq.client.mqtt.mqtt5.message.connect.connack.Mqtt5ConnAck;
import com.hivemq.client.mqtt.mqtt5.message.subscribe.suback.Mqtt5SubAck;
import com.hivemq.client.mqtt.mqtt5.message.subscribe.suback.Mqtt5SubAckReasonCode;
import lombok.Getter;
import org.correomqtt.business.exception.CorreoMqtt5SubscriptionFailed;
import org.correomqtt.business.exception.CorreoMqttConnectionFailedException;
import org.correomqtt.business.exception.CorreoMqttNotConnectedException;
import org.correomqtt.business.model.ConnectionConfigDTO;
import org.correomqtt.business.model.Lwt;
import org.correomqtt.business.model.MessageDTO;
import org.correomqtt.business.model.SubscriptionDTO;
import org.correomqtt.business.model.TlsSsl;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import javax.net.ssl.SSLException;
import java.time.LocalDateTime;
import java.time.ZoneOffset;
import java.util.List;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;
import java.util.function.Consumer;

@Getter
class CorreoMqtt5Client extends BaseCorreoMqttClient {

    private static final Logger LOGGER = LoggerFactory.getLogger(CorreoMqtt5Client.class);

    private Mqtt5BlockingClient mqtt5BlockingClient;

    CorreoMqtt5Client(ConnectionConfigDTO configDTO) {
        super(configDTO);
    }

    @Override
    Logger getLogger() {
        return LOGGER;
    }

    void executeConnect() throws SSLException, InterruptedException, ExecutionException, TimeoutException {

        ConnectionConfigDTO configDTO = getConfigDTO();

        Mqtt5ClientBuilder clientBuilder = MqttClient.builder()
                .useMqttVersion5()
                .identifier(configDTO.getClientId())
                .serverHost(configDTO.getUrl())
                .serverPort(getDestinationPort());

        if (configDTO.getSsl().equals(TlsSsl.KEYSTORE) && configDTO.getSslKeystore() != null && !configDTO.getSslKeystore().isEmpty()) {
            clientBuilder = clientBuilder
                    .sslConfig()
                    .keyManagerFactory(getKeyManagerFactory())
                    .trustManagerFactory(getTrustManagerFactory())
                    .applySslConfig();
        }

        clientBuilder.addDisconnectedListener(this);

        clientBuilder.addConnectedListener(this);

        mqtt5BlockingClient = clientBuilder.buildBlocking();

        Mqtt5ConnectBuilder.Send<CompletableFuture<Mqtt5ConnAck>> connBuilder = mqtt5BlockingClient
                .toAsync()
                .connectWith()
                .cleanStart(configDTO.isCleanSession())
                .keepAlive(10000);

        if (configDTO.getLwt().equals(Lwt.ON)) {
            connBuilder.willPublish()
                    .topic(configDTO.getLwtTopic())
                    .qos(configDTO.getLwtQoS().getMqttQos())
                    .payload(configDTO.getLwtPayload().getBytes())
                    .retain(configDTO.isLwtRetained())
                    .applyWillPublish();
        }

        if (configDTO.getUsername() != null && configDTO.getPassword() != null &&
                !configDTO.getUsername().isEmpty() && !configDTO.getPassword().isEmpty()) {
            connBuilder.simpleAuth()
                    .username(configDTO.getUsername())
                    .password(configDTO.getPassword().getBytes())
                    .applySimpleAuth();
        }

        Mqtt5ConnAck connAck = connBuilder.send().get(30, TimeUnit.SECONDS);
        if (connAck.getReasonCode().isError()) {
            closeIfConnectionExists();
            throw new CorreoMqttConnectionFailedException(connAck.getReasonCode());
        }
    }

    @Override
    void doReconnect(MqttClientDisconnectedContext context) {
        context.getReconnector()
                .reconnect(true)
                .delay(3000, TimeUnit.MILLISECONDS);
    }

    private synchronized void closeIfConnectionExists() {
        if (mqtt5BlockingClient != null && mqtt5BlockingClient.getState().isConnectedOrReconnect()) {
            mqtt5BlockingClient.disconnect();
        }
    }

    @Override
    void doUnsubscribe(SubscriptionDTO subscriptionDTO) {
        getCheckedClient().unsubscribeWith()
                .topicFilter(subscriptionDTO.getTopic())
                .send();
    }

    @Override
    void doPublish(MessageDTO messageDTO) {
        messageDTO.setDateTime(LocalDateTime.now(ZoneOffset.UTC));
        getCheckedClient().publishWith()
                .topic(messageDTO.getTopic())
                .payload(messageDTO.getPayload().getBytes())
                .qos(messageDTO.getQos().getMqttQos())
                .retain(messageDTO.isRetained())
                .send();
    }

    @Override
    void doSubscribe(SubscriptionDTO subscriptionDTO, Consumer<MessageDTO> incomingCallback)
            throws InterruptedException, ExecutionException, TimeoutException {

        Mqtt5SubAck subAck = getCheckedAsyncClient().subscribeWith()
                .topicFilter(subscriptionDTO.getTopic())
                .qos(subscriptionDTO.getQos().getMqttQos())
                .callback(mqtt5Publish -> incomingCallback.accept(new MessageDTO(mqtt5Publish)))
                .send()
                .get(10, TimeUnit.SECONDS);


        List<Mqtt5SubAckReasonCode> returnCodes = subAck.getReasonCodes();

        if (returnCodes.stream().anyMatch(Mqtt5SubAckReasonCode::isError)) {
            throw new CorreoMqtt5SubscriptionFailed(returnCodes);
        }
    }

    @Override
    void doDisconnect() {
        getCheckedClient().disconnect();
    }

    @Override
    boolean isConnected() {
        if (mqtt5BlockingClient == null) {
            return false;
        }
        return mqtt5BlockingClient.getState().isConnected();
    }

    private Mqtt5AsyncClient getCheckedAsyncClient() {
        return getCheckedClient().toAsync();
    }

    private Mqtt5BlockingClient getCheckedClient() {
        if (mqtt5BlockingClient == null) {
            throw new CorreoMqttNotConnectedException();
        }
        return mqtt5BlockingClient;
    }
}
