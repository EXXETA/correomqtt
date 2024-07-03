package org.correomqtt.gui.utils;

import org.correomqtt.di.Interceptor;

import java.lang.annotation.Documented;
import java.lang.annotation.ElementType;
import java.lang.annotation.Retention;
import java.lang.annotation.Target;

import static java.lang.annotation.ElementType.METHOD;
import static java.lang.annotation.RetentionPolicy.RUNTIME;

@Interceptor
@Target({ElementType.TYPE, METHOD})
@Retention(RUNTIME)
@Documented
public @interface FxThread {
}
