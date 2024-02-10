package org.correomqtt.di;

import java.lang.reflect.InvocationTargetException;
import java.util.function.Supplier;

@SingletonBean
public class SoyInterceptor {

    private SoyInterceptor() {
        // private constructor
    }

    public static Object intercept(Class<?> clazz, Supplier<Object> supplier) {
        InvocationContext ctx = new InvocationContext(supplier);
        InterceptorInfo interceptorInfo = SoyDi.getInterceptor(clazz);
        Object interceptor = SoyDi.inject(interceptorInfo.getClazz());
        try {
            return interceptorInfo.getAroundInvoke().invoke(interceptor, ctx);
        } catch (IllegalAccessException | InvocationTargetException e) {
            throw new SoyDiException("Failed to execute interceptor. ", e);
        }
    }
}
