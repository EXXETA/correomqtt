package org.correomqtt.business.dispatcher;

public class ShortcutDispatcher extends BaseConnectionDispatcher<ShortcutObserver> {

    private static ShortcutDispatcher instance;

    public static synchronized ShortcutDispatcher getInstance() {
        if (instance == null) {
            instance = new ShortcutDispatcher();
        }
        return instance;
    }

    public void onSubscriptionShortcutPressed(String uuidOfSelectedTab) {
        triggerFiltered(uuidOfSelectedTab, ShortcutObserver::onSubscriptionShortcutPressed);
    }

    public void onPublishShortcutPressed(String uuidOfSelectedTab){
        triggerFiltered(uuidOfSelectedTab, ShortcutObserver::onPublishShortcutPressed);
    }

    public void onClearIncomingShortcutPressed(String uuidOfSelectedTab){
        triggerFiltered(uuidOfSelectedTab, ShortcutObserver::onClearIncomingShortcutPressed);
    }

    public void onClearOutgoingShortcutPressed(String uuidOfSelectedTab){
        triggerFiltered(uuidOfSelectedTab, ShortcutObserver::onClearOutgoingShortcutPressed);
    }
}
