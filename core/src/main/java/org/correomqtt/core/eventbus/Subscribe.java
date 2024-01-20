package org.correomqtt.core.eventbus;

import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;

@Retention(RetentionPolicy.RUNTIME)
public @interface Subscribe {
    Class<? extends Event>[] value() default {};

    boolean sync() default false;
}
