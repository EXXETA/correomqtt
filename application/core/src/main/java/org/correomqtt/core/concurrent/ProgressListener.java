package org.correomqtt.core.concurrent;

@FunctionalInterface
public interface ProgressListener<P> {

    void progress(P progress);
}
