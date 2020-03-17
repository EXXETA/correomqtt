package com.exxeta.correomqtt.business.dispatcher;

public class PersistPublishHistoryDispatcher extends BaseConnectionDispatcher<PersistPublishHistoryObserver> {

    private static PersistPublishHistoryDispatcher instance;

    public static synchronized PersistPublishHistoryDispatcher getInstance() {
        if (instance == null) {
            instance = new PersistPublishHistoryDispatcher();
        }
        return instance;
    }

    public void updatedPublishes(String connectionId) {
        triggerFiltered(connectionId, o -> o.updatedPublishes(connectionId));
    }


    public void errorReadingPublishHistory(Throwable exception) {
        trigger(o -> o.errorReadingPublishHistory(exception));
    }

    public void errorWritingPublishHistory(Throwable exception) {
        trigger(o -> o.errorWritingPublishHistory(exception));
    }


}
