package org.correomqtt.di;

import java.lang.annotation.Documented;
import java.lang.annotation.Retention;
import java.lang.annotation.Target;

import static java.lang.annotation.ElementType.TYPE;
import static java.lang.annotation.RetentionPolicy.RUNTIME;

@Bean
@Target({ TYPE })
@Retention(RUNTIME)
@Documented
public @interface InterceptorBean {
    Class<?> value();
}
