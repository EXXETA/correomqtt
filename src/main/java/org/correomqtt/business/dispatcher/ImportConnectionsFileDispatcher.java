package org.correomqtt.business.dispatcher;

import org.correomqtt.business.model.ConnectionExportDTO;

import java.io.File;

public class ImportConnectionsFileDispatcher extends BaseDispatcher<ImportConnectionsFileObserver> {

    private static ImportConnectionsFileDispatcher instance;

    public static synchronized ImportConnectionsFileDispatcher getInstance() {
        if (instance == null) {
            instance = new ImportConnectionsFileDispatcher();
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
        trigger(ImportConnectionsFileObserver::onImportRunning);
    }

    public void onImportScheduled() {
        trigger(ImportConnectionsFileObserver::onImportScheduled);
    }

}
