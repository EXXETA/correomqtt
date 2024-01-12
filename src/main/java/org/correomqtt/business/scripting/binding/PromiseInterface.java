package org.correomqtt.business.scripting.binding;

import org.graalvm.polyglot.Value;

@FunctionalInterface
public interface PromiseInterface {
    void run(Value resolve, Value reject);
}
