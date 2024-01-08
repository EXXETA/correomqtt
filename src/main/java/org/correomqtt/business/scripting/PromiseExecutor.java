package org.correomqtt.business.scripting;

import org.graalvm.polyglot.Value;

@FunctionalInterface
public interface PromiseExecutor {
    void onPromiseCreation(Value onResolve, Value onReject);
}
