package org.correomqtt.business.dispatcher;

public interface PreloadingObserver extends BaseObserver {
    void onProgress(Double progress, String message);
}
