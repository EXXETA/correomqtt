package org.correomqtt.core.scripting.binding;

import org.correomqtt.di.DefaultBean;
import org.graalvm.polyglot.Context;
import org.graalvm.polyglot.HostAccess.Export;

import org.correomqtt.di.Inject;

@DefaultBean
public class ClientFactory {

    private final ClientImpl client;

    @Inject
    public ClientFactory(ClientImpl client) {
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
    }
}
