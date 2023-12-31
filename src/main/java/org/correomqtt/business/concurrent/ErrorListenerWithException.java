package org.correomqtt.business.concurrent;

@FunctionalInterface
public interface ErrorListenerWithException<E> {
    void error(E error, Throwable exception);
}
