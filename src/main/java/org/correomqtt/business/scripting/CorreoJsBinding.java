package org.correomqtt.business.scripting;

import org.correomqtt.business.model.ConnectionConfigDTO;
import org.correomqtt.business.model.MessageDTO;
import org.correomqtt.business.model.Qos;
import org.correomqtt.business.mqtt.CorreoMqttClient;
import org.correomqtt.business.mqtt.CorreoMqttClientFactory;
import org.correomqtt.business.mqtt.CorreoMqttClientState;
import org.correomqtt.business.utils.ConnectionHolder;
import org.correomqtt.business.utils.CorreoMqttConnection;
import org.graalvm.polyglot.HostAccess;

import javax.net.ssl.SSLException;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeoutException;

public class CorreoJsBinding {

    private final ExecutionDTO executionDTO;
    private String clientId;
    private CorreoMqttClient client;

    public CorreoJsBinding(ExecutionDTO executionDTO) {
        this.executionDTO = executionDTO;
        clientId = executionDTO.getScriptExecutionDTO().getExecutionId();

    }

    @HostAccess.Export
    public void sleep(long millis) {
        try {
            executionDTO.getOut().append("[Correo] Sleep for " + millis + "ms\n");
            Thread.sleep(millis);
        } catch (Exception e) {
            e.printStackTrace();
        }
    }

    @HostAccess.Export
    public String getConnectionId() {
        return executionDTO.getScriptExecutionDTO().getConnectionId();
    }

    @HostAccess.Export
    public void setClientId(String clientId){
        CorreoMqttConnection connection = ConnectionHolder.getInstance().getConnection(executionDTO.getScriptExecutionDTO().getConnectionId());
        if(connection.getConfigDTO().getClientId().equalsIgnoreCase(clientId)){
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
            executionDTO.getLog().append("[Correo] Publish on ")
                    .append(topic)
                    .append(" (qos:")
                    .append(qos)
                    .append(", retained:")
                    .append(isRetained ? "true" : "false")
                    .append(")\n");
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
        if(client.getState() == CorreoMqttClientState.DISCONNECTED){
            try {
                client.connect();
            } catch (InterruptedException | ExecutionException | TimeoutException | SSLException e) {
                // TODO fail
                return;
            }

            if(client.getState() != CorreoMqttClientState.CONNECTED){
                // TODO fail
            }
        }
    }

    private void ensureClient() {
        if (client == null) {
            CorreoMqttConnection connection = ConnectionHolder.getInstance().getConnection(executionDTO.getScriptExecutionDTO().getConnectionId());
            ConnectionConfigDTO connectiondDTO = new ConnectionConfigDTO(connection.getConfigDTO());
            connectiondDTO.setClientId(clientId);
            client = CorreoMqttClientFactory.createClient(connectiondDTO);
        }
    }

    @HostAccess.Export
    private void connect() {
        ensureClient();
        ensureConnected();
    }

    @HostAccess.Export
    private void disconnect(){
        if(client != null && client.getState() != CorreoMqttClientState.DISCONNECTED){
            client.disconnect(true);
        }
    }


}
