package org.correomqtt.business.concurrent;

@FunctionalInterface
public interface ProgressListener<P> {

    void progress(P progress);
}
