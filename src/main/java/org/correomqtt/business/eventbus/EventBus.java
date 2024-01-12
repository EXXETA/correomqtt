package org.correomqtt.business.eventbus;

import lombok.AllArgsConstructor;
import org.correomqtt.business.utils.FrontendBinding;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.lang.annotation.Annotation;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.lang.reflect.Parameter;
import java.util.Arrays;
import java.util.Collections;
import java.util.HashMap;
import java.util.HashSet;
import java.util.Map;
import java.util.Objects;
import java.util.Set;
import java.util.concurrent.CompletableFuture;
import java.util.function.Consumer;
import java.util.stream.Collectors;

public class EventBus {


    private static final Logger LOGGER = LoggerFactory.getLogger(EventBus.class);


    @AllArgsConstructor
    private static class Callback {
        private Object clazz;
        private Method method;
        private boolean withPayload;
        private boolean sync;
    }

    private EventBus() {
        // private constructor
    }

    private static final Map<Class<Event>, Set<Callback>> LISTENER = new HashMap<>();
    private static final Map<Object, HashMap<String, Method>> LISTENER_FILTER = new HashMap<>();
    private static final Map<Class<Event>, HashMap<String, Method>> EVENT_FILTER = new HashMap<>();

    public static void register(Object listener) {
        Arrays.stream(listener.getClass().getDeclaredMethods())
                .forEach(m -> {

                    EventSubscription annotationInfoFromMethod = getEventType(m);
                    EventSubscription annotationInfoFromParameter = getEventTypeFromParameter(m);

                    if (annotationInfoFromMethod != null && annotationInfoFromParameter != null) {
                        throw new IllegalArgumentException("Only one Subscribe annotation per method is allowed. Either at method level or at parameter level.");
                    }

                    Class<Event> eventType = null;
                    boolean sync = false;
                    if (annotationInfoFromMethod != null) {
                        eventType = annotationInfoFromMethod.event;
                        sync = annotationInfoFromMethod.sync;
                    } else if (annotationInfoFromParameter != null) {
                        eventType = annotationInfoFromParameter.event;
                        sync = annotationInfoFromParameter.sync;
                    }

                    if (eventType != null) {
                        LISTENER.computeIfAbsent(eventType, k -> new HashSet<>())
                                .add(new Callback(listener, m, m.getParameters().length > 0, sync));
                        registerEventType(eventType);
                    }

                    if (m.isAnnotationPresent(SubscribeFilter.class)) {
                        SubscribeFilter annotation = m.getAnnotation(SubscribeFilter.class);
                        String[] name = annotation.value();
                        Arrays.stream(name).forEach(n -> EventBus.LISTENER_FILTER
                                .computeIfAbsent(listener, k -> new HashMap<>())
                                .put(n, m));
                    }
                });
    }

    private static EventSubscription getEventType(Method m) {

        if (!m.isAnnotationPresent(Subscribe.class)) {
            return null;
        }

        Subscribe annotation = m.getAnnotation(Subscribe.class);
        Parameter[] params = m.getParameters();

        if (params.length > 1) {
            throw new IllegalArgumentException("Subscribe annotation allows no parameter or exactly one event parameter for methods");
        }

        Class<?> type;
        if (annotation.value().length == 1 && params.length == 0) {
            type = annotation.value()[0];
        } else if (annotation.value().length == 0 && params.length == 1) {
            type = params[0].getType();
        } else {
            throw new IllegalArgumentException("Subscribe annotations must have exactly one event definition. Either as annotation or method parameter.");
        }

        if (!Event.class.isAssignableFrom(type)) {
            throw new IllegalArgumentException("Type is not compatible to Event.");
        }

        @SuppressWarnings("unchecked")
        Class<Event> castedType = (Class<Event>) type;

        return new EventSubscription(castedType, annotation.sync());
    }

    private static void registerEventType(Class<Event> eventType) {
        if (EventBus.EVENT_FILTER.containsKey(eventType))
            return;

        Arrays.stream(eventType.getDeclaredMethods())
                .forEach(m -> {
                    // is filter
                    if (m.isAnnotationPresent(SubscribeFilter.class)) {
                        SubscribeFilter annotation = m.getAnnotation(SubscribeFilter.class);
                        String[] name = annotation.value();
                        Arrays.stream(name).forEach(n -> EventBus.EVENT_FILTER
                                .computeIfAbsent(eventType, k -> new HashMap<>())
                                .put(n, m));
                    }
                });
    }

    public static void unregister(Object listener) {
        EventBus.LISTENER.values().forEach(s -> s.removeIf(c -> c.clazz == listener));
    }

    private static EventSubscription getEventTypeFromParameter(Method m) {

        Annotation[][] annotatedParams = m.getParameterAnnotations();
        Subscribe annotation = (Subscribe) Arrays.stream(annotatedParams).map(a1 ->
                        Arrays.stream(a1)
                                .filter(a2 -> a2.annotationType().isAssignableFrom(Subscribe.class))
                                .findFirst()
                                .orElse(null))
                .filter(Objects::nonNull)
                .findFirst()
                .orElse(null);


        if (annotation == null)
            return null;

        if (annotatedParams.length != 1) {
            throw new UnsupportedOperationException("Event subscriber must have one parameter only.");
        }
        Parameter[] params = m.getParameters();
        if (!Event.class.isAssignableFrom(params[0].getType())) {
            throw new UnsupportedOperationException("@Subscribe must be used for parameters compatible with Event.");
        }

        //noinspection unchecked
        return new EventSubscription((Class<Event>) params[0].getType(), annotation.sync());

    }

    private static Set<Callback> getCallbacksToExecute(Event event) {
        Set<Callback> callbacks = LISTENER.get(event.getClass());
        if (callbacks == null)
            return Collections.emptySet();

        return callbacks.stream()
                .filter(c -> isValidEvent(event, c.clazz))
                .collect(Collectors.toSet());
    }

    private static void executeFire(Event event, Set<Callback> callbacks) {
        callbacks.forEach(c -> {
            if (c.sync) {
                executeMethod(c, event);
            } else {
                FrontendBinding.pushToFrontend(() -> {
                    executeMethod(c, event);
                });
            }
        });
    }

    private static void executeMethod(Callback c, Event event) {

        try {
            if (c.withPayload) {
                c.method.invoke(c.clazz, event);
            } else {
                c.method.invoke(c.clazz);
            }
        } catch (Exception e) {
            LOGGER.error("Unexpected Exception firing event. ", e);
            throw new EventCallbackExecutionException(e);
        }
    }

    public static int fire(Event event) {
        Set<Callback> callbacks = getCallbacksToExecute(event);
        executeFire(event, callbacks);
        return callbacks.size();
    }

    public static int fireAsync(Event event) {
        Set<Callback> callbacks = getCallbacksToExecute(event);
        CompletableFuture.runAsync(() -> executeFire(event, callbacks));
        return callbacks.size();
    }

    private static boolean isValidEvent(Event event, Object listener) {
        HashMap<String, Method> listenerFilter = LISTENER_FILTER.get(listener);
        HashMap<String, Method> eventFilter = EVENT_FILTER.get(event.getClass());

        // Either listener or event does not have filter -> is valid for sure
        if (listenerFilter == null || eventFilter == null)
            return true;

        // Iterate listener filter
        return listenerFilter.entrySet().stream().allMatch(lf -> {
            Method eventMethod = eventFilter.get(lf.getKey());

            // Event has filters, but no corresponding to this listener filter
            if (eventMethod == null)
                return true;

            // Event has a corresponding filter -> check if it matches.
            // The only way to invalidate an event is when both have the same filter but the content does not match.
            Method listenerMethod = lf.getValue();
            try {
                return eventMethod.invoke(event).equals(listenerMethod.invoke(listener));
            } catch (IllegalAccessException | InvocationTargetException e) {
                LOGGER.error("Unexpected Exception firing async event. ", e);
                throw new RuntimeException(e);
            }
        });
    }


    private record EventSubscription(Class<Event> event, boolean sync) {
    }
}
