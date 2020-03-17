package org.correomqtt.business.dispatcher;

import org.correomqtt.business.model.MessageDTO;

import java.io.File;

public interface ImportMessageObserver extends BaseConnectionObserver {
    void onImportSucceeded(MessageDTO messageDTO);

    void onImportCancelled(File file);

    void onImportFailed(File file, Throwable exception);

    default void onImportRunning() {
        // nothing to do
    }

    default void onImportScheduled() {
        // nothing to do
    }

    default void onImportStarted(File file) {
        // nothing to do
    }
}
