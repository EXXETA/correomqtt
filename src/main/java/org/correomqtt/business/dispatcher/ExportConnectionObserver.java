package org.correomqtt.business.dispatcher;

import java.io.File;

public interface ExportConnectionObserver extends BaseObserver {

    void onExportSucceeded();

    default void onExportCancelled(File file) {
        // nothing to do
    }

    default void onExportFailed(Throwable exception) {
        // noting to do
    }

    default void onExportRunning() {
        // nothing to do
    }

    default void onExportScheduled() {
        // nothing to do
    }

    default void onExportStarted() {
        // nothing to do
    }

}
