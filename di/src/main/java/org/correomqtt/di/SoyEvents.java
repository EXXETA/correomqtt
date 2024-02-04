package org.correomqtt.di;

@SingletonBean
public class SoyEvents {

    public int fire(Event event) {
        return EventBus.fire(event);
    }

    public int fireAsync(Event event) {
        return EventBus.fireAsync(event);
    }

    public static <T> void registerInstance(T instance) {
        EventBus.registerInstance(instance);
    }
}
