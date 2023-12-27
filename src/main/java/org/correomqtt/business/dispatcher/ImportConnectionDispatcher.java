package org.correomqtt.business.dispatcher;

import org.correomqtt.business.model.ConnectionExportDTO;

import java.io.File;

public class ImportConnectionDispatcher extends BaseDispatcher<ImportConnectionObserver> {

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

    public void onImportSucceeded( ConnectionExportDTO connectionExportDTO) {
        trigger(o -> o.onImportSucceeded(connectionExportDTO));
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
