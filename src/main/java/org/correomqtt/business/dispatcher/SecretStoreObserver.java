package org.correomqtt.business.dispatcher;


public interface SecretStoreObserver extends BaseObserver {
    void onPasswordFileUnreadable();

}