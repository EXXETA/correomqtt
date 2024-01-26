package org.correomqtt.core;

import dagger.Component;

import javax.inject.Singleton;


@Singleton
@Component
public abstract class CorreoCoreComponent {
    abstract CorreoCore core();
}
