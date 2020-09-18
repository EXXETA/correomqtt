package org.correomqtt.business.dispatcher;

public class ConfigDispatcher extends BaseDispatcher<ConfigObserver> {

    private static ConfigDispatcher instance;

    private ConfigDispatcher() {
    }

    public static synchronized ConfigDispatcher getInstance() {
        if (instance == null) {
            instance = new ConfigDispatcher();
        }
        return instance;
    }

    public void onConfigDirectoryEmpty() {
        trigger(ConfigObserver::onConfigDirectoryEmpty);
    }

    public void onConfigDirectoryNotAccessible() {
        trigger(ConfigObserver::onConfigDirectoryNotAccessible);
    }

    public void onAppDataNull() {
        trigger(ConfigObserver::onAppDataNull);
    }

    public void onUserHomeNull() {
        trigger(ConfigObserver::onUserHomeNull);
    }

    public void onFileAlreadyExists() {
        trigger(ConfigObserver::onFileAlreadyExists);
    }

    public void onInvalidPath() {
        trigger(ConfigObserver::onInvalidPath);
    }

    public void onInvalidJsonFormat() {
        trigger(ConfigObserver::onInvalidJsonFormat);
    }

    public void onSavingFailed(){trigger(ConfigObserver::onSavingFailed);}

    public void onConnectionsUpdated() { trigger(ConfigObserver::onConnectionsUpdated);}

    public void onSettingsUpdated(boolean showRestartRequiredDialog) { trigger(o -> o.onSettingsUpdated(showRestartRequiredDialog));}

    public void onConfigPrepareFailure() {
        trigger(ConfigObserver::onConfigPrepareFailed);
    }

}
