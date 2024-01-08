package org.correomqtt.business.scripting.binding;

import org.correomqtt.business.concurrent.ErrorListener;
import org.correomqtt.business.concurrent.ErrorListenerWithException;
import org.correomqtt.business.connection.ConnectTask;
import org.correomqtt.business.connection.ConnectionState;
import org.correomqtt.business.mqtt.CorreoMqttClient;
import org.correomqtt.business.scripting.PromiseExecutor;
import org.correomqtt.business.utils.ConnectionHolder;
import org.graalvm.polyglot.HostAccess;
import org.graalvm.polyglot.Value;

public class ClientConnect implements PromiseExecutor {


    private final String connectionId;

    public ClientConnect(String connectionId) {

        this.connectionId = connectionId;
    }

    @HostAccess.Export
    public void onPromiseCreation(Value resolve, Value reject) {
        try {
            CorreoMqttClient client = ConnectionHolder.getInstance().getClient(connectionId);
            if (client == null || client.getState() != ConnectionState.CONNECTED) {
                new ConnectTask(connectionId)
                        .onSuccess(ignore -> resolve.executeVoid("success"))
                        .onError((ErrorListener<Void>) ignore -> reject.executeVoid("fail"))
                        .run();

            }
            resolve.execute("success");
        } catch (Exception t) {
            reject.execute(t);
        }
    }
}
