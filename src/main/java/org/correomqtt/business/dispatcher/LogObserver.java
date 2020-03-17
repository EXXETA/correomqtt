package com.exxeta.correomqtt.business.dispatcher;

public interface LogObserver extends BaseObserver {
    void updateLog(String message);
}
