package org.correomqtt;

import com.sun.javafx.application.LauncherImpl;
import org.correomqtt.plugin.manager.PluginSecurityPolicy;

import java.security.Policy;

public class Launcher {
    public static void main(String[] args) {
        enablePluginSandbox();
        LauncherImpl.launchApplication(CorreoMqtt.class, CorreoPreloader.class, args);
    }

    private static void enablePluginSandbox() {
        Policy.setPolicy(new PluginSecurityPolicy());
        System.setSecurityManager(new SecurityManager());
    }
}
