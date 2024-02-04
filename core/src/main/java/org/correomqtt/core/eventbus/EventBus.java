package org.correomqtt.core.eventbus;

import lombok.AllArgsConstructor;
import org.correomqtt.core.utils.FrontendBinding;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import org.correomqtt.di.Inject;
import org.correomqtt.di.SingletonBean;
import java.lang.annotation.Annotation;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.lang.reflect.Parameter;
import java.util.Arrays;
import java.util.Collections;
import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Set;
import java.util.concurrent.CompletableFuture;
import java.util.stream.Collectors;

@SingletonBean
public class EventBus {


    private static final Logger LOGGER = LoggerFactory.getLogger(EventBus.class);
    private final Map<Class<Event>, Set<Callback>> listener = new HashMap<>();
    private final Map<Object, HashMap<String, Method>> listenerFilter = new HashMap<>();
    private final Map<Class<Event>, HashMap<String, Method>> eventFilter = new HashMap<>();

    @AllArgsConstructor
    private static class Callback {
        private Object clazz;
        private Method method;
        private boolean withPayload;
        private boolean sync;
    }

    private record EventSubscription(Class<Event> event, boolean sync) {
    }

    @Inject
    EventBus() {
        // private constructor
    }

    public void register(Object listener) {
        Arrays.stream(listener.getClass().getDeclaredMethods())
                .forEach(m -> {

                    List<EventSubscription> annotationInfoFromMethod = getEventType(m);
                    EventSubscription annotationInfoFromParameter = getEventTypeFromParameter(m);

                    List<EventSubscription> eventSubs;
                    if (annotationInfoFromMethod.isEmpty() && annotationInfoFromParameter == null) {
                        return;
                    } else if (!annotationInfoFromMethod.isEmpty()) {
                        eventSubs = annotationInfoFromMethod;
                    } else {
                        eventSubs = List.of(annotationInfoFromParameter);
                    }

                    eventSubs.forEach(es -> {
                        this.listener.computeIfAbsent(es.event, k -> new HashSet<>())
                                .add(new Callback(listener, m, m.getParameters().length > 0, es.sync));
                        registerEventType(es.event);
                    });

                    if (m.isAnnotationPresent(SubscribeFilter.class)) {
                        SubscribeFilter annotation = m.getAnnotation(SubscribeFilter.class);
                        String[] name = annotation.value();
                        Arrays.stream(name).forEach(n -> listenerFilter
                                .computeIfAbsent(listener, k -> new HashMap<>())
                                .put(n, m));
                    }
                });
    }

    private List<EventSubscription> getEventType(Method m) {

        if (!m.isAnnotationPresent(Subscribe.class)) {
            return Collections.emptyList();
        }

        Subscribe annotation = m.getAnnotation(Subscribe.class);
        Parameter[] params = m.getParameters();

        List<Class<?>> types;

        if (annotation.value().length == 1 && params.length >= 1) {
            throw new IllegalArgumentException("Subscribe annotation allows no parameter or exactly one event parameter for methods.");
        } else if (annotation.value().length > 1 && params.length > 0) {
            throw new IllegalArgumentException("If a Subscribe annotation is listening to multiple events no parameters are allowed.");
        } else if (annotation.value().length == 0 && params.length == 1) {
            types = List.of(params[0].getType());
        } else if (annotation.value().length == 0) {
            throw new IllegalStateException("Found annotation is not present.");
        } else {
            types = List.of(annotation.value());
        }

        return types.stream()
                .map(type -> {
                    if (!Event.class.isAssignableFrom(type)) {
                        throw new IllegalArgumentException("Type is not compatible to Event.");
                    }
                    //noinspection unchecked
                    return new EventSubscription((Class<Event>) type, annotation.sync());

                })
                .toList();

    }

    private EventSubscription getEventTypeFromParameter(Method m) {

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

    private void registerEventType(Class<Event> eventType) {
        if (eventFilter.containsKey(eventType))
            return;

        Arrays.stream(eventType.getDeclaredMethods())
                .forEach(m -> {
                    // is filter
                    if (m.isAnnotationPresent(SubscribeFilter.class)) {
                        SubscribeFilter annotation = m.getAnnotation(SubscribeFilter.class);
                        String[] name = annotation.value();
                        Arrays.stream(name).forEach(n -> eventFilter
                                .computeIfAbsent(eventType, k -> new HashMap<>())
                                .put(n, m));
                    }
                });
    }

    public void unregister(Object listener) {
        this.listener.values().forEach(s -> s.removeIf(c -> c.clazz == listener));
    }

    public int fire(Event event) {
        Set<Callback> callbacks = getCallbacksToExecute(event);
        executeFire(event, callbacks);
        return callbacks.size();
    }

    private Set<Callback> getCallbacksToExecute(Event event) {
        Set<Callback> callbacks = listener.get(event.getClass());
        if (callbacks == null)
            return Collections.emptySet();

        return callbacks.stream()
                .filter(c -> isValidEvent(event, c.clazz))
                .collect(Collectors.toSet());
    }

    private void executeFire(Event event, Set<Callback> callbacks) {
        callbacks.forEach(c -> {
            if (c.sync) {
                executeMethod(c, event);
            } else {
                FrontendBinding.pushToFrontend(() -> executeMethod(c, event));
            }
        });
    }

    private boolean isValidEvent(Event event, Object listener) {
        HashMap<String, Method> listenerFilter = this.listenerFilter.get(listener);
        HashMap<String, Method> eventFilter = this.eventFilter.get(event.getClass());

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
                throw new EventCallbackExecutionException(e);
            }
        });
    }

    private void executeMethod(Callback c, Event event) {

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

    public int fireAsync(Event event) {
        Set<Callback> callbacks = getCallbacksToExecute(event);
        CompletableFuture.runAsync(() -> executeFire(event, callbacks));
        return callbacks.size();
    }
}
