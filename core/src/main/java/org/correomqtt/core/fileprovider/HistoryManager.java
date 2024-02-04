package org.correomqtt.core.fileprovider;

import org.correomqtt.di.Inject;
import org.correomqtt.di.SingletonBean;
import java.util.Map;
import java.util.concurrent.ConcurrentHashMap;

@SingletonBean
public class HistoryManager {

    private final Map<String, PublishHistory> publishHistories = new ConcurrentHashMap<>();
    private final Map<String, PublishMessageHistory> publishMessageHistories = new ConcurrentHashMap<>();
    private final Map<String, SubscriptionHistory> subscriptionHistories = new ConcurrentHashMap<>();
    private final PublishHistoryFactory publishHistoryFactory;
    private final PublishMessageHistoryFactory publishMessageHistoryFactory;
    private final SubscriptionHistoryFactory subscriptionHistoryFactory;
    @Inject
    HistoryManager(PublishHistoryFactory publishHistoryFactory,
                   PublishMessageHistoryFactory publishMessageHistoryFactory,
                   SubscriptionHistoryFactory subscriptionHistoryFactory) {
        this.publishHistoryFactory = publishHistoryFactory;
        this.publishMessageHistoryFactory = publishMessageHistoryFactory;
        this.subscriptionHistoryFactory = subscriptionHistoryFactory;
    }

    public PublishHistory activatePublishHistory(String connectionId) {
        return publishHistories.computeIfAbsent(connectionId, k -> publishHistoryFactory.create(connectionId));
    }

    public PublishMessageHistory activatePublishMessageHistory(String connectionId) {
        return publishMessageHistories.computeIfAbsent(connectionId, k -> publishMessageHistoryFactory.create(connectionId));
    }

    public SubscriptionHistory activateSubscriptionHistory(String connectionId) {
        return subscriptionHistories.computeIfAbsent(connectionId, k -> subscriptionHistoryFactory.create(connectionId));
    }


    public void tearDownPublishHistory(String connectionId) {
        PublishHistory history = publishHistories.get(connectionId);
        if (history != null) {
            history.cleanUp();
            publishHistories.remove(connectionId);
        }
    }

    public void tearDownPublishMessageHistory(String connectionId) {

        PublishMessageHistory history = publishMessageHistories.get(connectionId);
        if (history != null) {
            history.cleanUp();
            publishMessageHistories.remove(connectionId);
        }
    }

    public void tearDownSubscriptionHistory(String connectionId) {

        SubscriptionHistory history = subscriptionHistories.get(connectionId);
        if (history != null) {
            history.cleanUp();
            subscriptionHistories.remove(connectionId);
        }
    }
}
