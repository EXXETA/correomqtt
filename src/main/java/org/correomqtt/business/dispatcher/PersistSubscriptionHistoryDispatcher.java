package com.exxeta.correomqtt.business.dispatcher;

public class PersistSubscriptionHistoryDispatcher extends BaseConnectionDispatcher<PersistSubscriptionHistoryObserver>{

    private static PersistSubscriptionHistoryDispatcher instance;

    public static synchronized PersistSubscriptionHistoryDispatcher getInstance() {
        if (instance == null) {
            instance = new PersistSubscriptionHistoryDispatcher();
        }
        return instance;
    }

    public void updatedSubscriptions(String connectionId) {
        triggerFiltered(connectionId, o -> o.updateSubscriptions(connectionId));
    }

    public void errorReadingSubscriptionHistory(Throwable exception) {
        trigger(o -> o.errorReadingSubscriptionHistory(exception));
    }

    public void errorWritingSubscriptionHistory(Throwable exception) {
        trigger(o -> o.errorWritingSubscriptionHistory(exception));
    }
}
