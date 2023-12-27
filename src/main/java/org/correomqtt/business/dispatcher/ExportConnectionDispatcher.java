package org.correomqtt.business.dispatcher;

import java.io.File;

public class ExportConnectionDispatcher extends BaseDispatcher<ExportConnectionObserver> {
    private static ExportConnectionDispatcher instance;

    public static synchronized ExportConnectionDispatcher getInstance() {
        if (instance == null) {
            instance = new ExportConnectionDispatcher();
        }
        return instance;
    }

    public void onExportSucceeded() {
        trigger(ExportConnectionObserver::onExportSucceeded);
    }

    public void onExportCancelled(File file) {
        trigger(o -> o.onExportCancelled(file));
    }

    public void onExportFailed(File file, Throwable exception) {
        trigger(o -> o.onExportFailed(exception));
    }

    public void onExportRunning() {
        trigger(ExportConnectionObserver::onExportRunning);
    }

    public void onExportScheduled() {
        trigger(ExportConnectionObserver::onExportScheduled);
    }

    public void onExportStarted() {
        trigger(ExportConnectionObserver::onExportStarted);
    }
}
