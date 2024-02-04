package org.correomqtt.di;

import lombok.AllArgsConstructor;
import lombok.Getter;

import java.lang.ref.WeakReference;
import java.lang.reflect.Method;
import java.util.Set;

@AllArgsConstructor
@Getter
class ObserverInfo {
    private final Set<WeakReference<?>> classes;
    private final Method method;
    private final boolean withPayload;
    private final boolean sync;
    private final boolean autocreate;
}
