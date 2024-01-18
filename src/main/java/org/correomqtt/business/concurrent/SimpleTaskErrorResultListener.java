package org.correomqtt.business.concurrent;

@FunctionalInterface
public interface SimpleTaskErrorResultListener {
    void error(SimpleTaskErrorResult errorResult);
}
