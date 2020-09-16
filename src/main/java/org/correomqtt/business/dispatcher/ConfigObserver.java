package org.correomqtt.business.dispatcher;

public interface ConfigObserver extends BaseObserver {
    void onConfigDirectoryEmpty();
    void onConfigDirectoryNotAccessible();
    void onAppDataNull();
    void onUserHomeNull();
    void onFileAlreadyExists();
    void onInvalidPath();
    void onInvalidJsonFormat();
    void onSavingFailed();
    void onSettingsUpdated(boolean showInfoDialog);
    void onConnectionsUpdated();
    void onConfigPrepareFailed();
    default void onPasswordSaveFailed(Exception e) {}
    default void onPasswordRetrievalFailed(Exception e) {}
}
