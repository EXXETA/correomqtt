package org.correomqtt.core.pubsub;

import lombok.Getter;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;

@Getter
@DefaultBean
public class PubSubTaskFactories {

    private final PublishTaskFactory publishFactory;
    private final SubscribeTaskFactory subscribeFactory;
    private final UnsubscribeTaskFactory unsubscribeFactory;

    @Inject
    public PubSubTaskFactories(PublishTaskFactory publishFactory,
                               SubscribeTaskFactory subscribeFactory,
                               UnsubscribeTaskFactory unsubscribeFactory) {
        this.publishFactory = publishFactory;
        this.subscribeFactory = subscribeFactory;
        this.unsubscribeFactory = unsubscribeFactory;
    }
}
