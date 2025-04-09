package org.correomqtt.di;

import lombok.AllArgsConstructor;
import lombok.Getter;

import java.lang.reflect.Constructor;
import java.util.List;

@Getter
@AllArgsConstructor
public class BeanInfo {
    private Class<?> clazz;
    private Constructor<?> constructor;
    private List<ParameterInfo> constructorParameters;
    private boolean singleton;
}
