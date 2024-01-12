package org.correomqtt.business.scripting.binding;

import org.graalvm.polyglot.HostAccess.Export;

public class PromiseClient {

    private final Client client;

    PromiseClient(Client client) {

        this.client = client;
    }

    @Export
    public PromiseInterface connect() {
        return (resolve, reject) -> client.connect(resolve::executeVoid, resolve::executeVoid);
    }
}
