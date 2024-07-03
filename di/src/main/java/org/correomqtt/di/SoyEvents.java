package org.correomqtt.di;

import java.lang.reflect.Type;

@SingletonBean
public class SoyEvents {

    public int fire(Event event) {
        return EventBus.fire(event);
    }

    public int fireAsync(Event event) {
        return EventBus.fireAsync(event);
    }

    public static <T> void registerInstance(T instance) {
        registerInstance(instance.getClass(), instance);
    }

    public static <T> void registerInstance(TypeReference<?> reference, T instance) {
        Type type = reference.getType();
        Class<T> rawType = (Class<T>) SoyDi.getRawType(type);
        EventBus.registerInstance(rawType, instance);
    }

    public static <T> void registerInstance(Class<?> clazz, T instance) {
        EventBus.registerInstance(clazz, instance);
    }
}
