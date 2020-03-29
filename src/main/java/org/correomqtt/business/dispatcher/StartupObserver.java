package org.correomqtt.business.dispatcher;

public interface StartupObserver extends BaseObserver {
    void onPluginUpdateFailed(String disabledPath);
    void onPluginLoadFailed();
}
