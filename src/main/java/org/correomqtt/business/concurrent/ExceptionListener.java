package org.correomqtt.business.concurrent;

@FunctionalInterface
public interface ExceptionListener {
    void error(Throwable ex);
}
