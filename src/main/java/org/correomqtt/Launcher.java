package org.correomqtt;

import java.util.PropertyPermission;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.slf4j.bridge.SLF4JBridgeHandler;

import static javafx.application.Application.launch;

public class Launcher {

    private static final Logger LOGGER = LoggerFactory.getLogger(Launcher.class);

    static {
        SLF4JBridgeHandler.install();
    }

    public static void main(String[] args) {
        try {

            //  java.security.AccessController.checkPermission(new PropertyPermission("org.graalvm.nativeimage.imagecode", "read"));
            //  System.setSecurityManager(new SecurityManager());

            // Loading lib secret keyring requires org.objectweb.asm, but including it clashes with lombok.
            // See: https://github.com/projectlombok/lombok/issues/2973
            // Workaround is to disable the use of asm library.
            System.setProperty("jnr.ffi.asm.enabled", "false");

            Thread.setDefaultUncaughtExceptionHandler((t, e) -> LOGGER.error("Uncaught Exception: ", e));

            System.setProperty("javafx.preloader", CorreoPreloader.class.getCanonicalName());

            launch(CorreoMqtt.class, args);
        } catch (Exception e) {
            LOGGER.error("Uncaught Main Exception: ", e);
        }
    }
}
