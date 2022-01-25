package org.correomqtt.business.dispatcher;

import org.correomqtt.business.provider.SecretStoreProvider;

public interface SecretStoreObserver extends BaseObserver {
    void onPasswordFileUnreadable();

}