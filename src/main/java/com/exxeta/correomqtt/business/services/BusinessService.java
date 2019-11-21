package com.exxeta.correomqtt.business.services;

public interface BusinessService {
    void onSucceeded();

    void onCancelled();

    void onFailed(Throwable exception);

    void onRunning();

    void onScheduled();
}
