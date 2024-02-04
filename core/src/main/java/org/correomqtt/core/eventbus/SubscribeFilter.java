package org.correomqtt.core.eventbus;


import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;

@Retention(RetentionPolicy.RUNTIME)
public @interface SubscribeFilter {

    String[] value() default {};
}
