package org.correomqtt.core.concurrent;

@FunctionalInterface
public interface TaskErrorResultListener<R> {
    void error(R errorResult);
}
