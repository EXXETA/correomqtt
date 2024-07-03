package org.correomqtt.di;

import lombok.AllArgsConstructor;
import lombok.Getter;

import java.lang.reflect.Method;

@Getter
@AllArgsConstructor
public class InterceptorInfo {
    private Class<?> clazz;
    private Method aroundInvoke;
}
