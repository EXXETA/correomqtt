package org.correomqtt.business.dispatcher;

import org.correomqtt.business.model.MessageDTO;

import java.io.File;

public class ExportMessageDispatcher extends BaseConnectionDispatcher<ExportMessageObserver> {

    private static ExportMessageDispatcher instance;

    public static synchronized ExportMessageDispatcher getInstance() {
        if (instance == null) {
            instance = new ExportMessageDispatcher();
        }
        return instance;
    }

    public void onExportStarted(String connectionId, File file, MessageDTO messageDTO) {
        triggerFiltered(connectionId, o -> o.onExportStarted(file, messageDTO));
    }

    public void onExportSucceeded(String connectionId) {
        triggerFiltered(connectionId, ExportMessageObserver::onExportSucceeded);
    }

    public void onExportCancelled(String connectionId, File file, MessageDTO messageDTO) {
        triggerFiltered(connectionId, o -> o.onExportCancelled(file, messageDTO));
    }

    public void onExportFailed(String connectionId, File file, MessageDTO messageDTO, Throwable exception) {
        triggerFiltered(connectionId, o -> o.onExportFailed(file, messageDTO, exception));
    }

    public void onExportRunning(String connectionId) {
        triggerFiltered(connectionId, ExportMessageObserver::onExportRunning);
    }

    public void onExportScheduled(String connectionId) {
        triggerFiltered(connectionId, ExportMessageObserver::onExportScheduled);
    }
}
