package org.correomqtt.di;


import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;

@Retention(RetentionPolicy.RUNTIME)
public @interface ObservesFilter {

    String[] value() default {};
}
