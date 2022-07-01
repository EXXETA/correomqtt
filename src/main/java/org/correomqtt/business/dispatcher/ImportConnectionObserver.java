package org.correomqtt.business.dispatcher;

import org.correomqtt.business.model.ConnectionConfigDTO;
import org.correomqtt.business.model.MessageDTO;

import java.io.File;
import java.util.List;

public interface ImportConnectionObserver extends BaseConnectionObserver{
    void onImportSucceeded(List<ConnectionConfigDTO> connectionConfigDTOS);

    void onImportCancelled(File file);

    void onImportFailed(File file, Throwable exception);

    default void onImportRunning() {
        // nothing to do
    }

    default void onImportScheduled() {
        // nothing to do
    }

    default void onImportStarted(String connectionId,File file) {
        // nothing to do
    }
}
