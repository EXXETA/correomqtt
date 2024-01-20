package org.correomqtt.core.mqtt;

import com.hivemq.client.mqtt.MqttClient;
import com.hivemq.client.mqtt.MqttClientSslConfigBuilder;
import com.hivemq.client.mqtt.lifecycle.MqttClientDisconnectedContext;
import com.hivemq.client.mqtt.mqtt3.Mqtt3AsyncClient;
import com.hivemq.client.mqtt.mqtt3.Mqtt3BlockingClient;
import com.hivemq.client.mqtt.mqtt3.Mqtt3ClientBuilder;
import com.hivemq.client.mqtt.mqtt3.message.Mqtt3ReturnCode;
import com.hivemq.client.mqtt.mqtt3.message.connect.Mqtt3ConnectBuilder;
import com.hivemq.client.mqtt.mqtt3.message.connect.connack.Mqtt3ConnAck;
import com.hivemq.client.mqtt.mqtt3.message.subscribe.suback.Mqtt3SubAck;
import com.hivemq.client.mqtt.mqtt3.message.subscribe.suback.Mqtt3SubAckReturnCode;
import lombok.Getter;
import org.correomqtt.core.exception.CorreoMqtt3SubscriptionFailed;
import org.correomqtt.core.exception.CorreoMqttConnectionFailedException;
import org.correomqtt.core.exception.CorreoMqttNotConnectedException;
import org.correomqtt.core.model.ConnectionConfigDTO;
import org.correomqtt.core.model.Lwt;
import org.correomqtt.core.model.MessageDTO;
import org.correomqtt.core.model.SubscriptionDTO;
import org.correomqtt.core.model.TlsSsl;
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
class CorreoMqtt3Client extends BaseCorreoMqttClient {

    private static final Logger LOGGER = LoggerFactory.getLogger(CorreoMqtt3Client.class);

    private Mqtt3BlockingClient mqtt3BlockingClient;

    CorreoMqtt3Client(ConnectionConfigDTO configDTO) {
        super(configDTO);
    }

    @Override
    Logger getLogger() {
        return LOGGER;
    }

    void executeConnect() throws SSLException, InterruptedException, ExecutionException, TimeoutException {

        ConnectionConfigDTO configDTO = getConfigDTO();

        Mqtt3ClientBuilder clientBuilder = MqttClient.builder()
                .useMqttVersion3()
                .identifier(configDTO.getClientId())
                .serverHost(configDTO.getUrl())
                .serverPort(getDestinationPort());

        if (configDTO.getSsl().equals(TlsSsl.KEYSTORE) && configDTO.getSslKeystore() != null && !configDTO.getSslKeystore().isEmpty()) {
            MqttClientSslConfigBuilder.Nested<? extends Mqtt3ClientBuilder> sslConfig = clientBuilder.sslConfig()
                    .keyManagerFactory(getKeyManagerFactory())
                    .trustManagerFactory(getTrustManagerFactory());

            if (!configDTO.isSslHostVerification()) {
                sslConfig = sslConfig.hostnameVerifier((s, sslSession) -> true);
            }

            clientBuilder = sslConfig.applySslConfig();
        }

        clientBuilder.addDisconnectedListener(this);

        clientBuilder.addConnectedListener(this);

        mqtt3BlockingClient = clientBuilder.buildBlocking();

        Mqtt3ConnectBuilder.Send<CompletableFuture<Mqtt3ConnAck>> connBuilder = mqtt3BlockingClient
                .toAsync()
                .connectWith()
                .cleanSession(configDTO.isCleanSession())
                .keepAlive(10);

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

        Mqtt3ConnAck connAck = connBuilder.send().get(10, TimeUnit.SECONDS);
        if (connAck.getReturnCode().isError()) {
            closeIfConnectionExists();
            throw new CorreoMqttConnectionFailedException(connAck.getReturnCode());
        }
    }

    @Override
    void doReconnect(MqttClientDisconnectedContext context) {
        context.getReconnector()
                .reconnect(true)
                .delay(3000, TimeUnit.MILLISECONDS);
    }

    private synchronized void closeIfConnectionExists() {
        if (mqtt3BlockingClient != null && mqtt3BlockingClient.getState().isConnectedOrReconnect()) {
            mqtt3BlockingClient.disconnect();
        }
    }

    @Override
    void doUnsubscribe(SubscriptionDTO subscriptionDTO) {
        getCheckedClient().unsubscribeWith()
                .topicFilter(subscriptionDTO.getTopic())
                .send();
    }

    @Override
    void doPublish(MessageDTO messageDTO)
            throws InterruptedException, ExecutionException, TimeoutException {

        messageDTO.setDateTime(LocalDateTime.now(ZoneOffset.UTC));
        getCheckedAsyncClient().publishWith()
                .topic(messageDTO.getTopic())
                .payload(messageDTO.getPayload().getBytes())
                .qos(messageDTO.getQos().getMqttQos())
                .retain(messageDTO.isRetained())
                .send()
                .get(10, TimeUnit.SECONDS);
    }

    @Override
    void doSubscribe(SubscriptionDTO subscriptionDTO, Consumer<MessageDTO> incomingCallback)
            throws InterruptedException, ExecutionException, TimeoutException {

        Mqtt3SubAck subAck = getCheckedAsyncClient().subscribeWith()
                .topicFilter(subscriptionDTO.getTopic())
                .qos(subscriptionDTO.getQos().getMqttQos())
                .callback(mqtt3Publish -> incomingCallback.accept(new MessageDTO(mqtt3Publish)))
                .send()
                .get(10, TimeUnit.SECONDS);

        List<Mqtt3SubAckReturnCode> returnCodes = subAck.getReturnCodes();

        if (returnCodes.stream().anyMatch(Mqtt3ReturnCode::isError)) {
            throw new CorreoMqtt3SubscriptionFailed(returnCodes);
        }
    }

    @Override
    void doDisconnect() {
        getCheckedClient().disconnect();
    }

    private Mqtt3AsyncClient getCheckedAsyncClient() {
        return getCheckedClient().toAsync();
    }

    private Mqtt3BlockingClient getCheckedClient() {
        if (mqtt3BlockingClient == null) {
            throw new CorreoMqttNotConnectedException();
        }

        return mqtt3BlockingClient;
    }
}
