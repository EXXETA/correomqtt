package org.correomqtt.di;

public interface TaskToFrontendPush {

    void pushToFrontend(Runnable runnable);
}
