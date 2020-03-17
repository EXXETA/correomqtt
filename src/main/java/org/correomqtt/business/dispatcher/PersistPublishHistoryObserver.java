package com.exxeta.correomqtt.business.dispatcher;

public interface PersistPublishHistoryObserver extends BaseConnectionObserver {
    void errorReadingPublishHistory(Throwable exception);
    void errorWritingPublishHistory(Throwable exception);
    void updatedPublishes(String connectionId);
}
