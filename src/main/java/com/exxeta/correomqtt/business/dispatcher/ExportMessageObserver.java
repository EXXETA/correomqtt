package com.exxeta.correomqtt.business.dispatcher;

import com.exxeta.correomqtt.business.model.MessageDTO;

import java.io.File;

public interface ExportMessageObserver extends BaseConnectionObserver {
    void onExportSucceeded();

    void onExportCancelled(File file, MessageDTO messageDTO);

    void onExportFailed(File file, MessageDTO messageDTO, Throwable exception);

    default void onExportRunning() {
        // nothing to do
    }

    default void onExportScheduled() {
        // nothing to do
    }

    default void onExportStarted(File file, MessageDTO messageDTO) {
        // nothing to do
    }
}
