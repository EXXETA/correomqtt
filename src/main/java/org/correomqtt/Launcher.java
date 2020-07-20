package org.correomqtt;

import com.sun.javafx.application.LauncherImpl;
import org.correomqtt.plugin.manager.PluginSecurityPolicy;
import java.security.Policy;
import java.util.PropertyPermission;

public class Launcher {
    public static void main(String[] args) {
        enablePluginSandbox();
        LauncherImpl.launchApplication(CorreoMqtt.class, CorreoPreloader.class, args);
    }

    private static void enablePluginSandbox() {
        Policy.setPolicy(new PluginSecurityPolicy());
        java.security.AccessController.checkPermission(new PropertyPermission("org.graalvm.nativeimage.imagecode", "read"));
        System.out.println("Perm granted");
        System.setSecurityManager(new SecurityManager());
    }
}
