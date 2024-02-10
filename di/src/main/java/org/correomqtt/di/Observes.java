package org.correomqtt.di;

import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;

@Retention(RetentionPolicy.RUNTIME)
public @interface Observes {
    Class<? extends Event>[] value() default {};

    boolean autocreate() default false;
}
