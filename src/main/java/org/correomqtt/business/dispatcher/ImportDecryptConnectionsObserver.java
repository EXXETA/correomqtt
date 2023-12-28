package org.correomqtt.business.dispatcher;

import org.correomqtt.business.model.ConnectionConfigDTO;
import org.correomqtt.business.model.ConnectionExportDTO;

import java.io.File;
import java.util.List;

public interface ImportDecryptConnectionsObserver extends BaseObserver{
    void onDecryptSucceeded(List<ConnectionConfigDTO> decryptedConnectionList);

    void onDecryptCancelled();

    void onDecryptFailed(Throwable exception);

    default void onDecryptRunning() {
        // nothing to do
    }

    default void onDecryptScheduled() {
        // nothing to do
    }

    default void onDecryptStarted() {
        // nothing to do
    }
}
