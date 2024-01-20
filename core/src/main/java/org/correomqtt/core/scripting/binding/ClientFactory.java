package org.correomqtt.core.scripting.binding;

import dagger.Component;
import org.graalvm.polyglot.HostAccess.Export;

public class ClientFactory {


    @Export
    public BlockingClient getBlockingClient() {
//        return new Client(null).toBlocking(); //TODO use factory
        return null;
    }


    @Export
    public AsyncClient getAsyncClient() {
        // return new Client(null).toAsync();  //TODO use factory
        return null;
    }

    @Export
    public PromiseClient getPromiseClient() {
//        return new Client(null).toPromise(); //TODO use factory
        return null;
    }
}
