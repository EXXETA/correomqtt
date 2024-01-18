package org.correomqtt.business.scripting.binding;

import org.graalvm.polyglot.HostAccess.Export;

public class ClientFactory {

    @Export
    public BlockingClient getBlockingClient() {
        return new Client().toBlocking();
    }


    @Export
    public AsyncClient getAsyncClient() {
        return new Client().toAsync();
    }

    @Export
    public PromiseClient getPromiseClient() {
        return new Client().toPromise();
    }
}
