package org.correomqtt.business.mqtt;

import net.schmizz.sshj.connection.channel.direct.Parameters;
import org.correomqtt.business.dispatcher.ConnectionLifecycleDispatcher;
import org.correomqtt.business.exception.CorreoMqttAlreadySubscribedException;
import org.correomqtt.business.exception.CorreoMqttNoRetriesLeftException;
import org.correomqtt.business.exception.CorreoMqttSshFailedException;
import org.correomqtt.business.model.Auth;
import org.correomqtt.business.model.ConnectionConfigDTO;
import org.correomqtt.business.model.MessageDTO;
import org.correomqtt.business.model.Proxy;
import org.correomqtt.business.model.SubscriptionDTO;
import com.hivemq.client.mqtt.lifecycle.MqttClientConnectedContext;
import com.hivemq.client.mqtt.lifecycle.MqttClientConnectedListener;
import com.hivemq.client.mqtt.lifecycle.MqttClientDisconnectedContext;
import com.hivemq.client.mqtt.lifecycle.MqttClientDisconnectedListener;
import com.hivemq.client.mqtt.lifecycle.MqttDisconnectSource;
import com.hivemq.client.util.KeyStoreUtil;
import net.schmizz.sshj.SSHClient;
import net.schmizz.sshj.connection.channel.direct.LocalPortForwarder;
import net.schmizz.sshj.transport.verification.PromiscuousVerifier;
import org.slf4j.Logger;
import org.slf4j.MarkerFactory;

import javax.net.ssl.KeyManagerFactory;
import javax.net.ssl.SSLException;
import javax.net.ssl.TrustManagerFactory;
import java.io.File;
import java.io.IOException;
import java.net.InetSocketAddress;
import java.net.ServerSocket;
import java.text.MessageFormat;
import java.util.HashSet;
import java.util.Set;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.function.Consumer;

abstract class BaseCorreoMqttClient implements CorreoMqttClient, MqttClientDisconnectedListener, MqttClientConnectedListener {

    private static final int MAX_RECONNECTS = 5;

    private final ConnectionConfigDTO configDTO;
    private final AtomicBoolean wasConnectedBefore = new AtomicBoolean(false);
    private final AtomicBoolean tryToReconnect = new AtomicBoolean(false);
    private final AtomicInteger triedReconnects = new AtomicInteger(0);
    private final Set<SubscriptionDTO> subscriptions = new HashSet<>();
    private SSHClient sshClient;
    private LocalPortForwarder localPortforwarder;

    protected BaseCorreoMqttClient(ConnectionConfigDTO configDTO) {
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

        if (configDTO.getProxy().equals(Proxy.SSH)) {
            try {
                setupSsh();
            } catch (IOException e) {
                disconnect(false);
                throw new CorreoMqttSshFailedException(
                        MessageFormat.format("{0} Error while creating ssh connection.", MarkerFactory.getMarker(configDTO.getName())),
                        e);
            }
        }

        getLogger().info(MarkerFactory.getMarker(configDTO.getName()), "Connecting to Broker using {}", configDTO.getMqttVersion().getDescription());
        executeConnect();
        wasConnectedBefore.set(true);
    }

    private void setupSsh() throws IOException, InterruptedException {

        getLogger().info(MarkerFactory.getMarker(configDTO.getName()), "Creating SSH tunnel to {}:{}.", configDTO.getSshHost(), configDTO.getPort());

        sshClient = new SSHClient();
        sshClient.addHostKeyVerifier(new PromiscuousVerifier());
        sshClient.connect(configDTO.getSshHost(), configDTO.getSshPort());

        if (configDTO.getAuth().equals(Auth.PASSWORD)) {
            sshClient.authPassword(
                    configDTO.getAuthUsername(),
                    configDTO.getAuthPassword());
        } else if (configDTO.getAuth().equals(Auth.KEYFILE)) {
            sshClient.authPublickey(
                    configDTO.getAuthUsername(),
                    configDTO.getAuthKeyfile());
        }

        final Parameters parameters
                = new Parameters("localhost", configDTO.getLocalPort(), configDTO.getUrl(), configDTO.getPort());
        Thread thread = new Thread(() -> {

            try{
                ServerSocket serverSocket = new ServerSocket();
                try (serverSocket) {
                    serverSocket.setReuseAddress(true);
                    serverSocket.bind(new InetSocketAddress(parameters.getLocalHost(), parameters.getLocalPort()));
                    localPortforwarder = sshClient.newLocalPortForwarder(parameters, serverSocket);
                    localPortforwarder.listen();
                }
            } catch (Exception e) {
                getLogger().error(MarkerFactory.getMarker(configDTO.getName()), "SSH socket to {}:{} failed.", configDTO.getSshHost(), configDTO.getPort());
                throw new CorreoMqttSshFailedException(e);
            }
        });

        thread.start();


        int sshRetries = 0;
        while (!sshClient.isConnected()) {
            if (sshRetries < 5) {
                sshRetries += 1;
                TimeUnit.SECONDS.sleep(1);
            } else {
                getLogger().error(MarkerFactory.getMarker(configDTO.getName()), "SSH tunnel to {}:{} failed.", configDTO.getSshHost(), configDTO.getPort());
                return;
            }
        }

        getLogger().info(MarkerFactory.getMarker(configDTO.getName()), "SSH tunnel to {}:{} established.", configDTO.getSshHost(), configDTO.getPort());

    }

    protected int getDestinationPort() {
        if (configDTO.getProxy().equals(Proxy.SSH)) {
            return configDTO.getLocalPort();
        } else {
            return configDTO.getPort();
        }
    }

    abstract void executeConnect() throws SSLException, InterruptedException, ExecutionException, TimeoutException;

    @Override
    public void onDisconnected(MqttClientDisconnectedContext context) {

        if (tryToReconnect.get()) {
            if (context.getSource() == MqttDisconnectSource.SERVER) {
                getLogger().info(MarkerFactory.getMarker(configDTO.getName()), "Disconnected by {}. Connection to broker lost.", context.getSource());
                reconnect(context);
            } else if (context.getSource() == MqttDisconnectSource.CLIENT) {
                getLogger().info(MarkerFactory.getMarker(configDTO.getName()), "Disconnected by {}. Connection to broker not possible.", context.getSource());
                reconnect(context);
            } else if (context.getSource() == MqttDisconnectSource.USER) {
                try {
                    localPortforwarder.close();
                    sshClient.disconnect();
                } catch (IOException e) {
                    e.printStackTrace();
                }
                getLogger().info(MarkerFactory.getMarker(configDTO.getName()), "Disconnected by {}. Connection to broker disconnected by user.", context.getSource());
            }
        }
    }

    @Override
    public void onConnected(MqttClientConnectedContext context) {
        triedReconnects.set(0);
        tryToReconnect.set(true);
        if (wasConnectedBefore.get()) {
            ConnectionLifecycleDispatcher.getInstance().onConnectionReconnected(configDTO.getId());
            getLogger().info(MarkerFactory.getMarker(configDTO.getName()), "Reconnected to broker successfully");
        }
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


    private synchronized void reconnect(MqttClientDisconnectedContext context) {
        getLogger().info(MarkerFactory.getMarker(configDTO.getName()), "Reconnecting connect to Broker.");
        if (tryToReconnect.get() && triedReconnects.get() < MAX_RECONNECTS && context.getSource() != MqttDisconnectSource.USER) {
            doReconnect(context);
            ConnectionLifecycleDispatcher.getInstance().onReconnectFailed(configDTO.getId(), triedReconnects, MAX_RECONNECTS);
            triedReconnects.incrementAndGet();
        } else {
            getLogger().error(MarkerFactory.getMarker(configDTO.getName()), "Maximum number of reconnects reached.");
            ConnectionLifecycleDispatcher.getInstance().onConnectionFailed(configDTO.getId(), new CorreoMqttNoRetriesLeftException());

        }
    }

    abstract void doReconnect(MqttClientDisconnectedContext context);


    @Override
    public synchronized void disconnect(boolean graceful) {

        //TODO use graceful

        tryToReconnect.set(false);

        if (isConnected()) {
            doDisconnect();
            getLogger().info(MarkerFactory.getMarker(configDTO.getName()), "Disconnected from broker.");
        } else {
            getLogger().info("Disconnecting client was not possible, cause was not connected.");
        }

        if (sshClient != null && sshClient.isConnected()) {
            getLogger().debug(MarkerFactory.getMarker(configDTO.getName()), "Disconnecting SSH tunnel for {}:{}.", configDTO.getSshHost(), configDTO.getPort());
            try {
                sshClient.close();
                getLogger().info(MarkerFactory.getMarker(configDTO.getName()), "SSH tunnel for {}:{} closed", configDTO.getSshHost(), configDTO.getPort());
            } catch (IOException e) {
                getLogger().error(MarkerFactory.getMarker(configDTO.getName()), "Disconnecting SSH tunnel for {}:{} failed", configDTO.getSshHost(), configDTO.getPort());
            }
            sshClient = null;
        }
    }

    abstract void doDisconnect();

    abstract boolean isConnected();

}
