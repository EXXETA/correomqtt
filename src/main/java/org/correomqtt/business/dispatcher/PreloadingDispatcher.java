package org.correomqtt.business.dispatcher;

public class PreloadingDispatcher extends BaseDispatcher<PreloadingObserver> {

    private static PreloadingDispatcher instance;

    public static synchronized PreloadingDispatcher getInstance() {
        if (instance == null) {
            instance = new PreloadingDispatcher();
        }
        return instance;
    }

    public void onProgress(double progress, String message) {
        trigger(o -> o.onProgress(progress, message));
    }

    public void onProgress(String message) {
        trigger(o -> o.onProgress(null, message));
    }

}
