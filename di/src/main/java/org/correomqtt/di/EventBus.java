package org.correomqtt.di;

import lombok.extern.slf4j.Slf4j;

import java.lang.annotation.Annotation;
import java.lang.ref.Reference;
import java.lang.ref.WeakReference;
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
import java.util.concurrent.CompletableFuture;

@Slf4j
class EventBus {

    // event class -> observer class : observer info (same as in OBSERVERS_TO_EVENT)
    private static final Map<Class<Event>, HashMap<Class<?>, ObserverInfo>> EVENT_TO_OBSERVERS = new HashMap<>();

    // observer class -> event class : observer info (same as in EVENT_TO_OBSERVERS)
    private static final Map<Class<?>, HashMap<Class<Event>, ObserverInfo>> OBSERVERS_TO_EVENT = new HashMap<>();

    // listener class -> filter identifier : filter method of observer
    private static final Map<Class<?>, HashMap<String, Method>> OBSERVERS_TO_FILTER = new HashMap<>();

    // event class -> filter identifier : filter method of event
    private static final Map<Class<Event>, HashMap<String, Method>> EVENT_TO_FILTER = new HashMap<>();

    private record EventObservers(Class<Event> event, boolean sync, boolean autocreate) {
    }

    EventBus() {
        // private constructor
    }

    public static void registerClass(Class<?> clazz) {
        log.debug("SoyEvents: Register class {} for events.", clazz);
        Arrays.stream(clazz.getDeclaredMethods())
                .forEach(m -> {
                    List<EventObservers> annotationInfoFromMethod = getEventType(m);
                    EventObservers annotationInfoFromParameter = getEventTypeFromParameter(m);
                    List<EventObservers> eventObservers;
                    if (annotationInfoFromMethod.isEmpty() && annotationInfoFromParameter == null) {
                        return;
                    } else if (!annotationInfoFromMethod.isEmpty()) {
                        eventObservers = annotationInfoFromMethod;
                    } else {
                        eventObservers = List.of(annotationInfoFromParameter);
                    }
                    eventObservers.forEach(es -> {
                        registerObservers(es, clazz, m);
                        registerEventType(es.event);
                    });
                    registerObserverFilter(clazz, m);
                });
    }

    private static List<EventObservers> getEventType(Method m) {
        if (!m.isAnnotationPresent(Observes.class)) {
            return Collections.emptyList();
        }
        Observes annotation = m.getAnnotation(Observes.class);
        List<Class<?>> types = getEventTypesFromMethodAndAnnotation(m, annotation);
        return types.stream()
                .map(type -> {
                    if (!Event.class.isAssignableFrom(type)) {
                        throw new IllegalArgumentException("Type is not compatible to Event.");
                    }
                    //noinspection unchecked
                    return new EventObservers((Class<Event>) type, annotation.sync(), annotation.autocreate());
                })
                .toList();
    }

    @SuppressWarnings("unchecked")
    private static EventObservers getEventTypeFromParameter(Method m) {
        Annotation[][] annotatedParams = m.getParameterAnnotations();
        Observes annotation = (Observes) Arrays.stream(annotatedParams).map(a1 ->
                        Arrays.stream(a1)
                                .filter(a2 -> a2.annotationType().isAssignableFrom(Observes.class))
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
            throw new UnsupportedOperationException("@Observes must be used for parameters compatible with Event.");
        }
        return new EventObservers((Class<Event>) params[0].getType(), annotation.sync(), annotation.autocreate());
    }

    private static void registerObservers(EventObservers es, Class<?> listenerClass, Method m) {
        log.debug("SoyEvents:  -> {} -> {}:{}", es.event, listenerClass, m);
        //TODO error if someone use autocreate with assisted constructor
        ObserverInfo observerInfo = new ObserverInfo(new HashSet<>(),
                m,
                m.getParameters().length > 0,
                es.sync,
                es.autocreate);
        EVENT_TO_OBSERVERS.computeIfAbsent(es.event, k -> new HashMap<>())
                .put(listenerClass, observerInfo);
        OBSERVERS_TO_EVENT.computeIfAbsent(listenerClass, k -> new HashMap<>())
                .put(es.event, observerInfo);
    }

    private static void registerEventType(Class<Event> eventType) {
        if (EVENT_TO_FILTER.containsKey(eventType))
            return;
        Arrays.stream(eventType.getDeclaredMethods())
                .forEach(m -> {
                    // is filter
                    if (m.isAnnotationPresent(ObservesFilter.class)) {
                        ObservesFilter annotation = m.getAnnotation(ObservesFilter.class);
                        String[] name = annotation.value();
                        Arrays.stream(name).forEach(n -> EVENT_TO_FILTER
                                .computeIfAbsent(eventType, k -> new HashMap<>())
                                .put(n, m));
                    }
                });
    }

    private static void registerObserverFilter(Class<?> listenerClass, Method m) {
        if (m.isAnnotationPresent(ObservesFilter.class)) {
            ObservesFilter annotation = m.getAnnotation(ObservesFilter.class);
            String[] name = annotation.value();
            Arrays.stream(name).forEach(n -> OBSERVERS_TO_FILTER
                    .computeIfAbsent(listenerClass, k -> new HashMap<>())
                    .put(n, m));
        }
    }

    private static List<Class<?>> getEventTypesFromMethodAndAnnotation(Method m, Observes annotation) {
        Parameter[] params = m.getParameters();
        List<Class<?>> types;
        if (annotation.value().length == 1 && params.length >= 1) {
            throw new IllegalArgumentException("@Observes annotation allows no parameter or exactly one event parameter for methods.");
        } else if (annotation.value().length > 1 && params.length > 0) {
            throw new IllegalArgumentException("If a @Observes annotation is listening to multiple events no parameters are allowed.");
        } else if (annotation.value().length == 0 && params.length == 1) {
            types = List.of(params[0].getType());
        } else if (annotation.value().length == 0) {
            throw new IllegalStateException("Found annotation is not present.");
        } else {
            types = List.of(annotation.value());
        }
        return types;
    }

    public static <T> void registerInstance(T instance) {
        log.debug("SoyEvents: Register instance for events {}", instance);
        Class<?> clazz = instance.getClass();
        if (!OBSERVERS_TO_EVENT.containsKey(clazz)) {
            return;
        }
        HashMap<Class<Event>, ObserverInfo> observers = OBSERVERS_TO_EVENT.get(clazz);
        observers.values().forEach(oi -> oi.getClasses().add(new WeakReference<>(instance)));
    }

    public static int fire(Event event) {
        if (event.isLogable()) {
            log.debug("SoyEvents: Fire event {}", event.getClass());
        }
        List<ObserverInfo> observerInfos = getCallbacksToExecute(event);
        executeFire(event, observerInfos);
        return observerInfos.size();
    }

    public static int fireAsync(Event event) {
        if (event.isLogable()) {
            log.debug("SoyEvents: Fire async event {}", event.getClass());
        }
        List<ObserverInfo> observerInfos = getCallbacksToExecute(event);
        CompletableFuture.runAsync(() -> executeFire(event, observerInfos));
        return observerInfos.size();
    }

    private static List<ObserverInfo> getCallbacksToExecute(Event event) {
        Class<? extends Event> eventClass = event.getClass();
        if (!EVENT_TO_OBSERVERS.containsKey(eventClass)) {
            return Collections.emptyList();
        }
        HashMap<Class<?>, ObserverInfo> observerInfos = EVENT_TO_OBSERVERS.get(eventClass);
        // autocreate instances
        observerInfos.forEach((key, oi) -> {
            if (oi.getClasses().isEmpty() && oi.isAutocreate()) {
                SoyDi.inject(key);
            }
        });
        // collect observers
        return observerInfos.entrySet().stream()
                .filter(oiEntry -> isValidEvent(event, oiEntry.getKey()))
                .map(Map.Entry::getValue)
                .toList();
    }

    private static void executeFire(Event event, List<ObserverInfo> observerInfos) {
        observerInfos.forEach(oi -> {
            //TODO sync
            executeMethod(oi, event);
        });
    }

    private static boolean isValidEvent(Event event, Class<?> observer) {
        HashMap<String, Method> listenerFilter = OBSERVERS_TO_FILTER.get(observer);
        HashMap<String, Method> eventFilter = EVENT_TO_FILTER.get(event.getClass());
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
                return eventMethod.invoke(event).equals(listenerMethod.invoke(observer));
            } catch (IllegalAccessException | InvocationTargetException e) {
                throw new EventCallbackExecutionException(e);
            }
        });
    }

    private static void executeMethod(ObserverInfo oi, Event event) {
        oi.getClasses()
                .stream()
                .map(Reference::get)
                .filter(Objects::nonNull)
                .forEach(c -> {
                    try {
                        if (oi.isWithPayload()) {
                            oi.getMethod().invoke(c, event);
                        } else {
                            oi.getMethod().invoke(c);
                        }
                        if (event.isLogable()) {
                            log.debug("SoyEvents: Sent {} to {}:{}", event.getClass(), c, oi.getMethod());
                        }
                    } catch (Exception e) {
                        log.error("Unexpected Exception firing event. ", e);
                        throw new EventCallbackExecutionException(e);
                    }
                });
    }
}
