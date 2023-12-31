package org.correomqtt.business.concurrent;

@FunctionalInterface
public interface SuccessListener<T> {
    void success(T result);
}
