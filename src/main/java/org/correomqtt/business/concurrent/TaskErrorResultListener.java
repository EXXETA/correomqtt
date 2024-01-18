package org.correomqtt.business.concurrent;

@FunctionalInterface
public interface TaskErrorResultListener<R> {
    void error(R errorResult);
}
