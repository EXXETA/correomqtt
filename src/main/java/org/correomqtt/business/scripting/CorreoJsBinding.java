package org.correomqtt.business.scripting;

import org.correomqtt.business.connection.ConnectTask;
import org.correomqtt.business.connection.DisconnectTask;
import org.correomqtt.business.model.ConnectionConfigDTO;
import org.correomqtt.business.model.MessageDTO;
import org.correomqtt.business.model.Qos;
import org.correomqtt.business.mqtt.CorreoMqttClient;
import org.correomqtt.business.mqtt.CorreoMqttClientFactory;
import org.correomqtt.business.connection.ConnectionState;
import org.correomqtt.business.utils.ConnectionHolder;
import org.correomqtt.business.utils.CorreoMqttConnection;
import org.graalvm.polyglot.HostAccess;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import javax.net.ssl.SSLException;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeoutException;

public class CorreoJsBinding {

    private static final Logger LOGGER = LoggerFactory.getLogger(ScriptingBackend.class);

    private final ExecutionContextDTO executionContextDTO;
    private String clientId;
    private CorreoMqttClient client;

    CorreoJsBinding(ExecutionContextDTO executionContextDTO) {
        this.executionContextDTO = executionContextDTO;
        clientId = executionContextDTO.getExecutionDTO().getExecutionId();

    }

    @HostAccess.Export
    public void sleep(long millis) {
        try {
            executionContextDTO.getOut().write(("[Correo] Sleep for " + millis + "ms\n").getBytes(StandardCharsets.UTF_8));
            Thread.sleep(millis);
        } catch (IOException e) {
            throw new BindingException(e);
        } catch (InterruptedException e) {
            LOGGER.debug("Correo JS Binding interrupted correo::sleep");
            Thread.currentThread().interrupt();
        }
    }

    @HostAccess.Export
    public String getConnectionId() {
        return executionContextDTO.getExecutionDTO().getConnectionId();
    }

    @HostAccess.Export
    public void setClientId(String clientId) {
        CorreoMqttConnection connection = ConnectionHolder.getInstance().getConnection(executionContextDTO.getExecutionDTO().getConnectionId());
        if (connection.getConfigDTO().getClientId().equalsIgnoreCase(clientId)) {
            // TODO fail
            return;
        }

        this.clientId = clientId;
    }

    @HostAccess.Export
    public void publish(String topic) {
        publish(topic, 0, false, "");
    }

    @HostAccess.Export
    public void publish(String topic, int qos) {
        publish(topic, qos, false, "");
    }

    @HostAccess.Export
    public void publish(String topic, int qos, boolean isRetained) {
        publish(topic, qos, isRetained, "");
    }

    @HostAccess.Export
    public void publish(String topic, int qos, boolean isRetained, String payload) {
        try {
         /*   executionContextDTO.getLog().append("[Correo] Publish on ")
                    .append(topic)
                    .append(" (qos:")
                    .append(qos)
                    .append(", retained:")
                    .append(isRetained ? "true" : "false")
                    .append(")\n"); // TODO pipe */
            ensureConnectedClient();
            client.publish(MessageDTO.builder()
                    .topic(topic)
                    .qos(Qos.fromJsonValue(qos))
                    .isRetained(isRetained)
                    .payload(payload)
                    .build());
        } catch (Exception e) {
            //TODO fail
        }
    }

    private void ensureConnectedClient() {
        ensureClient();
        ensureConnected();
    }

    private void ensureConnected() {
        if (client.getState() == ConnectionState.DISCONNECTED_GRACEFUL) {
            try {
                client.connect();
            } catch (InterruptedException | ExecutionException | TimeoutException | SSLException e) {
                // TODO fail
                return;
            }

            if (client.getState() != ConnectionState.CONNECTED) {
                // TODO fail
            }
        }
    }

    private void ensureClient() {
    }

    @HostAccess.Export
    public void connect() {
        new ConnectTask(executionContextDTO.getExecutionDTO().getConnectionId())
                .run();
        try {
            executionContextDTO.getOut().write(("[Correo] Connected to Broker\n").getBytes(StandardCharsets.UTF_8));
        } catch (IOException e) {
            throw new RuntimeException(e);
        }
    }

    @HostAccess.Export
    public void disconnect() {
        new DisconnectTask(executionContextDTO.getExecutionDTO().getConnectionId())
                .run();
        try {
            executionContextDTO.getOut().write(("[Correo] Disconnected from broker\n").getBytes(StandardCharsets.UTF_8));
        } catch (IOException e) {
            throw new RuntimeException(e);
        }
    }


}
