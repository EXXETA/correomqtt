package org.correomqtt.business.scripting.binding;

import org.graalvm.polyglot.HostAccess.Export;

import java.util.function.Consumer;

public class AsyncClient {

    private final Client client;

    AsyncClient(Client client) {
        this.client = client;
    }

    @Export
    void connect(Runnable resolve, Consumer<Throwable> reject) {
        client.connect(resolve, reject);
    }

    @Export
    void disconnect(Runnable resolve, Consumer<Throwable> reject) {
        client.disconnect(resolve, reject);
    }
}
