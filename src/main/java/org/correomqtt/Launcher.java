package org.correomqtt;

import org.correomqtt.plugin.manager.PluginSecurityPolicy;

import java.security.Policy;

import static javafx.application.Application.launch;

public class Launcher {
    public static void main(String[] args) {
        enablePluginSandbox();
        System.setProperty("javafx.preloader", CorreoPreloader.class.getCanonicalName());
        launch(CorreoMqtt.class, args);
    }

    private static void enablePluginSandbox() {
        Policy.setPolicy(new PluginSecurityPolicy());
        System.setSecurityManager(new SecurityManager());
    }
}
