package org.correomqtt.core.concurrent;

@FunctionalInterface
public interface SuccessListener<T> {
    void success(T result);
}
