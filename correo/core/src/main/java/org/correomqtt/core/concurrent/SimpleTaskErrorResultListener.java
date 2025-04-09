package org.correomqtt.core.concurrent;

@FunctionalInterface
public interface SimpleTaskErrorResultListener {
    void error(SimpleTaskErrorResult errorResult);
}
