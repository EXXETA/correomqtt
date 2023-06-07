package org.correomqtt;

import static javafx.application.Application.launch;

public class Launcher {
    public static void main(String[] args) {
        System.setProperty("javafx.preloader", CorreoPreloader.class.getCanonicalName());

        // Loading lib secret keyring requires org.objectweb.asm, but including it clashes with lombok.
        // See: https://github.com/projectlombok/lombok/issues/2973
        // Workaround is to disable the use of asm library.
        System.setProperty("jnr.ffi.asm.enabled", "false");

        launch(CorreoMqtt.class, args);
    }
}
