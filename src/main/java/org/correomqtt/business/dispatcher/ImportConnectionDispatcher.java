package org.correomqtt.business.dispatcher;

import org.correomqtt.business.model.ConnectionConfigDTO;
import org.correomqtt.business.model.MessageDTO;

import java.io.File;
import java.util.List;

public class ImportConnectionDispatcher extends BaseConnectionDispatcher<ImportConnectionObserver>{

    private static ImportConnectionDispatcher instance;

    public static synchronized ImportConnectionDispatcher getInstance() {
        if (instance == null) {
            instance = new ImportConnectionDispatcher();
        }
        return instance;
    }

    public void onImportStarted(String connectionId, File file) {
        triggerFiltered(connectionId, o -> o.onImportStarted(connectionId,file));
    }

    public void onImportSucceeded(String connectionId, List<ConnectionConfigDTO> connectionConfigDTOS) {
        triggerFiltered(connectionId, o -> o.onImportSucceeded(connectionConfigDTOS));
    }

    public void onImportCancelled(String connectionId, File file) {
        triggerFiltered(connectionId, o -> o.onImportCancelled(file));
    }

    public void onImportFailed(String connectionId, File file, Throwable exception) {
        triggerFiltered(connectionId, o -> o.onImportFailed(file, exception));
    }

    public void onImportRunning(String connectionId) {
        triggerFiltered(connectionId, ImportConnectionObserver::onImportRunning);
    }

    public void onImportScheduled(String connectionId) {
        triggerFiltered(connectionId, ImportConnectionObserver::onImportScheduled);
    }

}
