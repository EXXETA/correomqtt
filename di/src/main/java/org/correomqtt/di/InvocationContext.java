package org.correomqtt.di;

import lombok.AllArgsConstructor;
import lombok.Getter;

import java.util.function.Supplier;

@Getter
@AllArgsConstructor
public class InvocationContext {

    private Supplier<Object> supplier;

    public Object proceed() {
        return supplier.get();
    }
}
