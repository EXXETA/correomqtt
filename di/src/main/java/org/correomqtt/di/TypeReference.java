package org.correomqtt.di;

import lombok.Getter;
import lombok.NonNull;

import java.lang.reflect.ParameterizedType;
import java.lang.reflect.Type;

/*
 * Basically taken from Jackson
 * Code: https://github.com/FasterXML/jackson-core/blob/2.17/src/main/java/com/fasterxml/jackson/core/type/TypeReference.java
 * License: https://github.com/FasterXML/jackson-core/blob/2.17/LICENSE
 */

@Getter
public abstract class TypeReference<T> implements Comparable<TypeReference<T>> {
    protected final Type type;

    protected TypeReference() {
        Type superClass = this.getClass().getGenericSuperclass();
        if (superClass instanceof Class) {
            throw new SoyDiException("TypeReference constructed without actual type information.");
        } else {
            this.type = ((ParameterizedType) superClass).getActualTypeArguments()[0];
        }
    }

    @Override
    public int hashCode() {
        return type.hashCode();
    }

    @Override
    public boolean equals(Object o) {
        return o == this;
    }

    @Override
    public int compareTo(@NonNull TypeReference<T> o) {
        return Integer.compare(o.hashCode(), hashCode());
    }
}

