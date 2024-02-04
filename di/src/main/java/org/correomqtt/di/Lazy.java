package org.correomqtt.di;

public class Lazy<T> {

    private final Class<T> clazz;
    private final TypeReference<T> reference;

    public Lazy(Class<T> clazz) {
        this.clazz = clazz;
        this.reference = null;
    }

    public Lazy(TypeReference<T> reference) {
        this.clazz = null;
        this.reference = reference;
    }

    public T get() {
        if (clazz != null) {
            return SoyDi.inject(clazz);
        } else {
            return SoyDi.inject(reference);
        }
    }
}
