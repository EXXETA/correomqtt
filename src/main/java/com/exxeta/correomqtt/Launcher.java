package com.exxeta.correomqtt;

import com.exxeta.correomqtt.plugin.manager.PluginSecurityPolicy;

import java.security.Policy;

public class Launcher {
    public static void main(String[] args) {
        enablePluginSandbox();
        CorreoMqtt.main(args);
    }

    private static void enablePluginSandbox() {
        Policy.setPolicy(new PluginSecurityPolicy());
        System.setSecurityManager(new SecurityManager());
    }
}
