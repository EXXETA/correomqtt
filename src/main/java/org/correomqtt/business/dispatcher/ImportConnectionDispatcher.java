package org.correomqtt.business.dispatcher;

import org.correomqtt.business.model.ConnectionConfigDTO;

import java.io.File;
import java.util.List;

public class ImportConnectionDispatcher extends BaseConnectionDispatcher<ImportConnectionObserver> {

    private static ImportConnectionDispatcher instance;

    public static synchronized ImportConnectionDispatcher getInstance() {
        if (instance == null) {
            instance = new ImportConnectionDispatcher();
        }
        return instance;
    }

    public void onImportStarted(File file) {
        trigger(o -> o.onImportStarted(file));
    }

    public void onImportSucceeded( List<ConnectionConfigDTO> connectionConfigDTOS) {
        trigger(o -> o.onImportSucceeded(connectionConfigDTOS));
    }

    public void onImportCancelled( File file) {
        trigger(o -> o.onImportCancelled(file));
    }

    public void onImportFailed( File file, Throwable exception) {
        trigger(o -> o.onImportFailed(file, exception));
    }

    public void onImportRunning() {
        trigger(ImportConnectionObserver::onImportRunning);
    }

    public void onImportScheduled() {
        trigger(ImportConnectionObserver::onImportScheduled);
    }

}
