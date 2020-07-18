package org.correomqtt.business.scripting;

import org.correomqtt.business.model.MessageDTO;
import org.correomqtt.business.model.Qos;
import org.correomqtt.business.model.ScriptExecutionDTO;
import org.correomqtt.business.utils.ConnectionHolder;
import org.correomqtt.business.utils.CorreoMqttConnection;
import org.graalvm.polyglot.HostAccess;

import java.io.IOException;
import java.util.concurrent.Callable;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeoutException;
import java.util.function.Function;

public class CorreoJsBinding {

    private final CorreoMqttConnection connection;
    private final ScriptExecutionDTO scriptExecutionDTO;

    public CorreoJsBinding(ScriptExecutionDTO scriptExecutionDTO) {
        this.scriptExecutionDTO = scriptExecutionDTO;
        connection = ConnectionHolder.getInstance().getConnection(scriptExecutionDTO.getConnectionId());
        //TODO validate connection?
    }

    @HostAccess.Export
    public void sleep(long millis) {
        try {
            scriptExecutionDTO.getOut().write(("[Correo] Sleep for " + millis + "ms\n").getBytes());
            Thread.sleep(millis);
        } catch (Exception e) {
            e.printStackTrace();
        }
    }

    @HostAccess.Export
    public String getConnectionId() {
        return scriptExecutionDTO.getConnectionId();
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
            scriptExecutionDTO.getOut().write(("[Correo] Publish on " + topic + " (qos:" + qos + ", retained:" + (isRetained ? "true" : "false") + ")\n").getBytes());
            connection.getClient().publish(MessageDTO.builder()
                    .topic(topic)
                    .qos(Qos.fromJsonValue(qos))
                    .isRetained(isRetained)
                    .payload(payload)
                    .build());
        } catch (Exception e) {
            e.printStackTrace();
        }
    }


}
