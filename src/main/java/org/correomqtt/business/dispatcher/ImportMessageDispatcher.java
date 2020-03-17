package com.exxeta.correomqtt.business.dispatcher;

import com.exxeta.correomqtt.business.model.MessageDTO;

import java.io.File;

public class ImportMessageDispatcher extends BaseConnectionDispatcher<ImportMessageObserver> {

    private static ImportMessageDispatcher instance;

    public static synchronized ImportMessageDispatcher getInstance() {
        if (instance == null) {
            instance = new ImportMessageDispatcher();
        }
        return instance;
    }

    public void onImportStarted(String connectionId, File file) {
        triggerFiltered(connectionId, o -> o.onImportStarted(file));
    }

    public void onImportSucceeded(String connectionId, MessageDTO messageDTO) {
        triggerFiltered(connectionId, o -> o.onImportSucceeded(messageDTO));
    }

    public void onImportCancelled(String connectionId, File file) {
        triggerFiltered(connectionId, o -> o.onImportCancelled(file));
    }

    public void onImportFailed(String connectionId, File file, Throwable exception) {
        triggerFiltered(connectionId, o -> o.onImportFailed(file, exception));
    }

    public void onImportRunning(String connectionId) {
        triggerFiltered(connectionId, ImportMessageObserver::onImportRunning);
    }

    public void onImportScheduled(String connectionId) {
        triggerFiltered(connectionId, ImportMessageObserver::onImportScheduled);
    }
}
