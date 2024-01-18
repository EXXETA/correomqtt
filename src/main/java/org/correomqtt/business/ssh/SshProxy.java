package org.correomqtt.business.ssh;

import net.schmizz.sshj.SSHClient;
import net.schmizz.sshj.connection.channel.direct.LocalPortForwarder;
import net.schmizz.sshj.connection.channel.direct.Parameters;
import net.schmizz.sshj.transport.verification.PromiscuousVerifier;
import org.correomqtt.business.exception.CorreoMqttSshFailedException;
import org.correomqtt.business.model.Auth;
import org.correomqtt.business.model.ConnectionConfigDTO;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.slf4j.MarkerFactory;

import java.io.IOException;
import java.net.InetSocketAddress;
import java.net.ServerSocket;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.TimeUnit;

public class SshProxy {

    private static final Logger LOGGER = LoggerFactory.getLogger(SshProxy.class);

    private final SshProxyDelegate delegate;
    private final ConnectionConfigDTO configDTO;
    private SSHClient sshClient;

    private LocalPortForwarder localPortforwarder;

    public SshProxy(SshProxyDelegate delegate, ConnectionConfigDTO configDTO) {
        this.delegate = delegate;
        this.configDTO = configDTO;
    }

    public void connect() {
        try {

            LOGGER.info(MarkerFactory.getMarker(configDTO.getName()), "Creating SSH tunnel to {}:{}.", configDTO.getSshHost(), configDTO.getPort());

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

            forkConnection();

            int currentWait = 0;
            int maxWait = 5;
            while (!sshClient.isConnected()) {
                if (currentWait < maxWait) {
                    currentWait += 1;
                    LOGGER.debug(MarkerFactory.getMarker(configDTO.getName()), "SSH tunnel not connected yet. Waiting {}s/{}s", currentWait, maxWait);
                    TimeUnit.SECONDS.sleep(1);
                } else {
                    delegate.onProxyFailed();
                    LOGGER.error(MarkerFactory.getMarker(configDTO.getName()), "SSH tunnel to {}:{} failed.", configDTO.getSshHost(), configDTO.getPort());
                    return;
                }
            }

            LOGGER.info(MarkerFactory.getMarker(configDTO.getName()), "SSH tunnel to {}:{} established. Local Port: {}", configDTO.getSshHost(), configDTO.getPort(), configDTO.getLocalPort());
        } catch (IOException e) {
            delegate.onProxyFailed();
            LOGGER.error(MarkerFactory.getMarker(configDTO.getName()), "SSH connection to {}:{} failed.", configDTO.getSshHost(), configDTO.getPort(), e);
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            delegate.onProxyFailed();
            LOGGER.error(MarkerFactory.getMarker(configDTO.getName()), "SSH connection to {}:{} failed.", configDTO.getSshHost(), configDTO.getPort(), e);
        }
    }

    private void forkConnection() {
        final Parameters parameters = new Parameters(
                "localhost",
                configDTO.getLocalPort(),
                configDTO.getUrl(),
                configDTO.getPort());

        CompletableFuture.runAsync(() -> {
            LOGGER.debug(MarkerFactory.getMarker(configDTO.getName()), "Forking SSH connection to {}:{}", configDTO.getSshHost(), configDTO.getSshPort());
            try (ServerSocket serverSocket = new ServerSocket()) {
                serverSocket.setReuseAddress(true);
                serverSocket.bind(new InetSocketAddress(parameters.getLocalHost(), parameters.getLocalPort()));
                localPortforwarder = sshClient.newLocalPortForwarder(parameters, serverSocket);
                localPortforwarder.listen();
            } catch (Exception e) {
                LOGGER.error(MarkerFactory.getMarker(configDTO.getName()), "SSH socket to {}:{} failed.", configDTO.getSshHost(), configDTO.getPort());
                delegate.onProxyFailed();
                throw new CorreoMqttSshFailedException(e);
            } finally {
                try {
                    if (localPortforwarder != null) {
                        localPortforwarder.close();
                    }
                    if (sshClient != null) {
                        sshClient.disconnect();
                        sshClient.close();
                    }
                } catch (IOException e) {
                    LOGGER.warn(MarkerFactory.getMarker(configDTO.getName()), "SSH Tunnel closed unsuccessful.", e);
                }
            }
        });
    }


    public void disconnect(String source) {
        try {
            localPortforwarder.close();
            sshClient.disconnect();
            LOGGER.info(MarkerFactory.getMarker(configDTO.getName()), "Disconnected SSH by {}.", source);
        } catch (IOException e) {
            LOGGER.warn(MarkerFactory.getMarker(configDTO.getName()), "SSH Tunnel disconnecting unsuccessful.", e);
        }
    }

    public int getPort() {
        return configDTO.getLocalPort();
    }
}
