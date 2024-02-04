package org.correomqtt.di;

import java.lang.reflect.ParameterizedType;
import java.lang.reflect.Type;

// copied from Jackson
public abstract class TypeReference<T> implements Comparable<TypeReference<T>> {
    protected final Type _type;

    protected TypeReference() {
        Type superClass = this.getClass().getGenericSuperclass();
        if (superClass instanceof Class) {
            throw new RuntimeException("Internal error: TypeReference constructed without actual type information");
        } else {
            this._type = ((ParameterizedType) superClass).getActualTypeArguments()[0];
        }
    }

    public Type getType() {
        return this._type;
    }

    public int compareTo(TypeReference<T> o) {
        return 0;
    }
}

