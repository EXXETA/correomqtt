package com.exxeta.correomqtt.business.dispatcher;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.HashSet;
import java.util.Set;
import java.util.function.Consumer;

public abstract class BaseDispatcher<T extends BaseObserver> {

    private static final Logger LOGGER = LoggerFactory.getLogger(BaseConnectionDispatcher.class);

    protected Set<T> observer = new HashSet<>();

    public void addObserver(T observer) {
        this.observer.add(observer);
    }

    public void removeObserver(T observer) {
        this.observer.remove(observer);
    }

    void trigger(Consumer<T> trigger) {

        final String callerString = getCallerString();

        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug("Trigger: {}", callerString);
        }

        observer.forEach(o -> {
            if (LOGGER.isTraceEnabled()) {
                LOGGER.trace("Trigger: {} -> {}", callerString, o.getClass().getSimpleName());
            }
            trigger.accept(o);
        });
    }


    String getCallerString() {
        if (LOGGER.isDebugEnabled()) {
            StackTraceElement st = Thread.currentThread().getStackTrace()[3];
            String fullClassName = st.getClassName();
            String simpleClassName = fullClassName.substring(fullClassName.lastIndexOf('.') + 1);
            return simpleClassName + "." + st.getMethodName() + ":" + st.getLineNumber();
        }
        return null;
    }

}
