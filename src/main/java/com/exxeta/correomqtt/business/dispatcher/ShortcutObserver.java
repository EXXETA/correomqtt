package com.exxeta.correomqtt.business.dispatcher;

public interface ShortcutObserver extends BaseConnectionObserver {

    default void onSubscriptionShortcutPressed(){};
    default void onPublishShortcutPressed(){};
    default void onClearOutgoingShortcutPressed(){};
    default void onClearIncomingShortcutPressed(){};


}
