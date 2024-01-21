package org.correomqtt.core.pubsub;

import lombok.Getter;

import javax.inject.Inject;

@Getter
public class PubSubTaskFactories {

    private final PublishTask.Factory publishFactory;
    private final SubscribeTask.Factory subscribeFactory;
    private final UnsubscribeTask.Factory unsubscribeFactory;

    @Inject
    public PubSubTaskFactories(PublishTask.Factory publishFactory,
                               SubscribeTask.Factory subscribeFactory,
                               UnsubscribeTask.Factory unsubscribeFactory) {
        this.publishFactory = publishFactory;
        this.subscribeFactory = subscribeFactory;
        this.unsubscribeFactory = unsubscribeFactory;
    }
}
