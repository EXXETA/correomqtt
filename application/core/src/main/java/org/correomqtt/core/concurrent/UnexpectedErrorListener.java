package org.correomqtt.core.concurrent;

@FunctionalInterface
public interface UnexpectedErrorListener {
    void error(Throwable throwable);
}
