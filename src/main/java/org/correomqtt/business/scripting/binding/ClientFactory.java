package org.correomqtt.business.scripting.binding;

import org.graalvm.polyglot.HostAccess.Export;

public class ClientFactory {

    @Export
    public static BlockingClient getBlockingClient() {
        return new Client().toBlocking();
    }


    @Export
    public static AsyncClient getAsyncClient() {
        return new Client().toAsync();
    }

    @Export
    public static PromiseClient getPromiseClient() {
        return new Client().toPromise();
    }
}
