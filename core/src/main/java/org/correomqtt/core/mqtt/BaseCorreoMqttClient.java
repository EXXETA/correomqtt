package org.correomqtt.core.mqtt;

import com.hivemq.client.mqtt.lifecycle.MqttClientConnectedContext;
import com.hivemq.client.mqtt.lifecycle.MqttClientConnectedListener;
import com.hivemq.client.mqtt.lifecycle.MqttClientDisconnectedContext;
import com.hivemq.client.mqtt.lifecycle.MqttClientDisconnectedListener;
import com.hivemq.client.mqtt.lifecycle.MqttDisconnectSource;
import com.hivemq.client.util.KeyStoreUtil;
import lombok.Getter;
import org.correomqtt.core.connection.ConnectionState;
import org.correomqtt.core.connection.ConnectionStateChangedEvent;
import org.correomqtt.di.SoyEvents;
import org.correomqtt.core.exception.CorreoMqttAlreadySubscribedException;
import org.correomqtt.core.model.ConnectionConfigDTO;
import org.correomqtt.core.model.MessageDTO;
import org.correomqtt.core.model.Proxy;
import org.correomqtt.core.model.SubscriptionDTO;
import org.correomqtt.core.ssh.SshProxy;
import org.correomqtt.core.ssh.SshProxyDelegate;
import org.slf4j.Logger;
import org.slf4j.MarkerFactory;

import javax.net.ssl.KeyManagerFactory;
import javax.net.ssl.SSLException;
import javax.net.ssl.TrustManagerFactory;
import java.io.File;
import java.util.HashSet;
import java.util.Set;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeoutException;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.function.Consumer;
import java.util.function.Function;
import java.util.function.Supplier;

abstract class BaseCorreoMqttClient implements CorreoMqttClient, MqttClientDisconnectedListener, MqttClientConnectedListener, SshProxyDelegate {

    private static final int MAX_RECONNECTS = 5;

    private final SoyEvents soyEvents;
    private final ConnectionConfigDTO configDTO;
    private final AtomicInteger triedReconnects = new AtomicInteger(0);
    private final Set<SubscriptionDTO> subscriptions = new HashSet<>();

    @Getter
    private SshProxy proxy;

    @Getter
    private ConnectionState state = ConnectionState.DISCONNECTED_GRACEFUL;

    protected BaseCorreoMqttClient(SoyEvents soyEvents,
                                   ConnectionConfigDTO configDTO) {
        this.soyEvents = soyEvents;
        this.configDTO = configDTO;
    }

    ConnectionConfigDTO getConfigDTO() {
        return configDTO;
    }

    abstract Logger getLogger();

    public Set<SubscriptionDTO> getSubscriptions() {
        return new HashSet<>(subscriptions);
    }

    @Override
    public synchronized void connect() throws InterruptedException, ExecutionException, TimeoutException, SSLException {
        changeState(ConnectionState.CONNECTING);
        executeConditionallyOnSshProxy(SshProxy::connect);
        getLogger().info(MarkerFactory.getMarker(configDTO.getName()), "Connecting to Broker using {}", configDTO.getMqttVersion().getDescription());
        executeConnect();
    }


    abstract void executeConnect() throws SSLException, InterruptedException, ExecutionException, TimeoutException;

    @Override
    public void onDisconnected(MqttClientDisconnectedContext context) {

        changeState(ConnectionState.DISCONNECTING);

        if (context.getSource() == MqttDisconnectSource.USER) {
            executeConditionallyOnSshProxy(sshProxy -> sshProxy.disconnect(context.getSource().toString()));
            changeState(ConnectionState.DISCONNECTED_GRACEFUL);
            getLogger().info("Disconnected by {}", context.getSource());
            return;
        }

        if (getLogger().isInfoEnabled()) {
            getLogger().info(MarkerFactory.getMarker(configDTO.getName()), "Disconnected by {}. Connection to broker lost.", context.getSource());
        }

        if (triedReconnects.get() < MAX_RECONNECTS) {
            triedReconnects.incrementAndGet();
            getLogger().info(MarkerFactory.getMarker(configDTO.getName()), "Reconnecting to Broker {}/{}", triedReconnects.get(), MAX_RECONNECTS);
            changeState(ConnectionState.RECONNECTING);
            doReconnect(context);
        } else {
            getLogger().error(MarkerFactory.getMarker(configDTO.getName()), "Maximum number of reconnects reached {}/{}.", triedReconnects.get(), MAX_RECONNECTS);
            executeConditionallyOnSshProxy(sshProxy -> sshProxy.disconnect(context.getSource().toString()));
            changeState(ConnectionState.DISCONNECTED_UNGRACEFUL);
        }

    }


    @Override
    public void onConnected(MqttClientConnectedContext context) {
        triedReconnects.set(0);
        getLogger().info(MarkerFactory.getMarker(configDTO.getName()), "Successfully connected to broker");
        this.changeState(ConnectionState.CONNECTED);
    }

    @Override
    public void onProxyFailed() {
        disconnect();
        getLogger().error(MarkerFactory.getMarker(configDTO.getName()), "Proxy failed");
    }

    protected int getDestinationPort() {
        return getConditionallyOnSshProxy(SshProxy::getPort, configDTO::getPort);
    }


    KeyManagerFactory getKeyManagerFactory() throws SSLException {
        return KeyStoreUtil.keyManagerFromKeystore(
                new File(configDTO.getSslKeystore()),
                configDTO.getSslKeystorePassword(),
                configDTO.getSslKeystorePassword());
    }

    TrustManagerFactory getTrustManagerFactory() throws SSLException {
        return KeyStoreUtil.trustManagerFromKeystore(
                new File(configDTO.getSslKeystore()),
                configDTO.getSslKeystorePassword());

    }

    @Override
    public synchronized void unsubscribe(SubscriptionDTO subscriptionDTO) {
        doUnsubscribe(subscriptionDTO);
        subscriptions.remove(subscriptionDTO);
    }

    abstract void doUnsubscribe(SubscriptionDTO subscriptionDTO);

    @Override
    public synchronized void publish(MessageDTO messageDTO) throws InterruptedException, ExecutionException, TimeoutException {
        doPublish(messageDTO);
    }

    abstract void doPublish(MessageDTO messageDTO) throws InterruptedException, ExecutionException, TimeoutException;


    abstract void doSubscribe(SubscriptionDTO subscriptionDTO, Consumer<MessageDTO> incomingCallback)
            throws InterruptedException, ExecutionException, TimeoutException;

    @Override
    public synchronized void subscribe(SubscriptionDTO subscriptionDTO, Consumer<MessageDTO> incomingCallback)
            throws InterruptedException, ExecutionException, TimeoutException {
        if (subscriptions.contains(subscriptionDTO)) {
            throw new CorreoMqttAlreadySubscribedException(getConfigDTO().getId(), subscriptionDTO);
        }
        doSubscribe(subscriptionDTO, incomingCallback);
        subscriptions.add(subscriptionDTO);
    }


    @Override
    public synchronized void disconnect() {
        doDisconnect();
        executeConditionallyOnSshProxy(sshProxy -> sshProxy.disconnect(MqttDisconnectSource.USER.toString()));
        getLogger().info(MarkerFactory.getMarker(configDTO.getName()), "Disconnected from broker.");
    }

    abstract void doReconnect(MqttClientDisconnectedContext context);

    abstract void doDisconnect();

    private void changeState(ConnectionState state) {
        this.state = state;
        if (state == ConnectionState.DISCONNECTED_UNGRACEFUL) {
            getLogger().error(MarkerFactory.getMarker(configDTO.getName()), "Connection state changed to {}", state, new RuntimeException());
        } else {
            getLogger().info(MarkerFactory.getMarker(configDTO.getName()), "Connection state changed to {}", state);
        }
        soyEvents.fireAsync(new ConnectionStateChangedEvent(getConfigDTO().getId(),
                state,
                triedReconnects.get(),
                MAX_RECONNECTS));
    }

    /* SSH Proxy Helper */

    private void executeConditionallyOnSshProxy(Consumer<SshProxy> sshProxyCallback) {
        getConditionallyOnSshProxy(sshProxy -> {
            sshProxyCallback.accept(sshProxy);
            return null;
        }, () -> null);
    }

    private <T> T getConditionallyOnSshProxy(Function<SshProxy, T> sshProxyCallback, Supplier<T> insteadCallback) {
        if (!configDTO.getProxy().equals(Proxy.SSH)) {
            return insteadCallback.get();
        }

        if (proxy == null) {
            proxy = new SshProxy(this, configDTO);
        }

        return sshProxyCallback.apply(proxy);
    }

}
