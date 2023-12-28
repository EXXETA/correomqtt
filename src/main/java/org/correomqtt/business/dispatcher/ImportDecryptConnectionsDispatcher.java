package org.correomqtt.business.dispatcher;

import org.correomqtt.business.model.ConnectionConfigDTO;

import java.util.List;

public class ImportDecryptConnectionsDispatcher extends BaseDispatcher<ImportDecryptConnectionsObserver> {

    private static ImportDecryptConnectionsDispatcher instance;

    public static synchronized ImportDecryptConnectionsDispatcher getInstance() {
        if (instance == null) {
            instance = new ImportDecryptConnectionsDispatcher();
        }
        return instance;
    }

    public void onDecryptStarted() {
        trigger(ImportDecryptConnectionsObserver::onDecryptStarted);
    }

    public void onDecryptSucceeded(List<ConnectionConfigDTO> decryptedConnectionList) {
        trigger(o -> o.onDecryptSucceeded(decryptedConnectionList));
    }

    public void onDecryptCancelled() {
        trigger(ImportDecryptConnectionsObserver::onDecryptCancelled);
    }

    public void onDecryptFailed( Throwable exception) {
        trigger(o -> o.onDecryptFailed(exception));
    }

    public void onDecryptRunning() {
        trigger(ImportDecryptConnectionsObserver::onDecryptRunning);
    }

    public void onDecryptScheduled() {
        trigger(ImportDecryptConnectionsObserver::onDecryptScheduled);
    }

}
