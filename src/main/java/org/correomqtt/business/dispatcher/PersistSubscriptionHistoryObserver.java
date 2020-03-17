package org.correomqtt.business.dispatcher;

public interface PersistSubscriptionHistoryObserver extends BaseConnectionObserver {
    void updateSubscriptions(String connectionId);
    void errorReadingSubscriptionHistory(Throwable exception);
    void errorWritingSubscriptionHistory(Throwable exception);
}
