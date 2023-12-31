package org.correomqtt.business.concurrent;

@FunctionalInterface
public interface UnexpectedErrorListener {
    void error(Throwable throwable);
}
