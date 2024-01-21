package org.correomqtt;

import ch.qos.logback.classic.LoggerContext;
import ch.qos.logback.classic.joran.JoranConfigurator;
import ch.qos.logback.core.joran.spi.JoranException;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.slf4j.bridge.SLF4JBridgeHandler;

import java.io.IOException;
import java.io.InputStream;

import static javafx.application.Application.launch;

public class Launcher {

    private static final Logger LOGGER = LoggerFactory.getLogger(Launcher.class);

    static {
        SLF4JBridgeHandler.install();
    }

    public static void main(String[] args) {
        try {

            setLoggerFilePath();

            // Loading lib secret keyring requires org.objectweb.asm, but including it clashes with lombok.
            // See: https://github.com/projectlombok/lombok/issues/2973
            // Workaround is to disable the use of asm library.
            System.setProperty("jnr.ffi.asm.enabled", "false");

            Thread.setDefaultUncaughtExceptionHandler((t, e) -> LOGGER.error("Uncaught Exception: ", e));

            System.setProperty("javafx.preloader", CorreoPreloader.class.getCanonicalName());

            launch(FxApplication.class, args);
        } catch (Exception e) {
            LOGGER.error("Uncaught Main Exception: ", e);
        }
    }


    private static void setLoggerFilePath() {
        // Set the path for file logging to user directory.
        // TODO  System.setProperty("correomqtt-logfile", SettingsProvider.getInstance().getLogPath());

        LoggerContext loggerContext = (LoggerContext) LoggerFactory.getILoggerFactory();
        loggerContext.reset();
        JoranConfigurator configurator = new JoranConfigurator();
        try (InputStream configStream = CorreoPreloader.class.getResourceAsStream("logger-config.xml")) {
            configurator.setContext(loggerContext);
            configurator.doConfigure(configStream);
        } catch (JoranException | IOException e) {
            System.out.println("Problem configuring logger: " + e.getMessage());
        }
    }


}
