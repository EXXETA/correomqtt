package org.correomqtt.business.concurrent;

@FunctionalInterface
public interface ErrorListener<E> {
    void error(E error);
}
