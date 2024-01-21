package org.correomqtt.core.scripting.binding;

import org.graalvm.polyglot.Context;
import org.graalvm.polyglot.HostAccess.Export;

import javax.inject.Inject;

public class ClientFactory {

    private final Client client;
    private Context context;
    @Inject
    public ClientFactory(Client client) {
        this.client = client;
    }

    @Export
    public BlockingClient getBlockingClient() {
        return client.toBlocking();
    }


    @Export
    public AsyncClient getAsyncClient() {
        return client.toAsync();
    }

    @Export
    public PromiseClient getPromiseClient() {
        return client.toPromise();
    }

    public void setContext(Context context) {
        this.client.setContext(context);
        this.context = context;
    }
}
