package org.correomqtt.business.dispatcher;

import org.correomqtt.business.provider.SecretStoreProvider;

public class SecretStoreDispatcher extends BaseDispatcher<SecretStoreObserver> {

    private static SecretStoreDispatcher instance;

    private SecretStoreDispatcher() {
        // private constructor
    }

    public static synchronized SecretStoreDispatcher getInstance() {
        if (instance == null) {
            instance = new SecretStoreDispatcher();
        }
        return instance;
    }

    public void onPasswordFileUnreadable() {
        observer.forEach(o -> o.onPasswordFileUnreadable());
    }

}