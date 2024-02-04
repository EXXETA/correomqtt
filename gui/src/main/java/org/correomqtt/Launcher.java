package org.correomqtt;

import ch.qos.logback.classic.LoggerContext;
import ch.qos.logback.classic.joran.JoranConfigurator;
import ch.qos.logback.core.joran.spi.JoranException;
import org.correomqtt.preloader.PreloaderImpl;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.slf4j.bridge.SLF4JBridgeHandler;

import java.io.IOException;
import java.io.InputStream;

import static javafx.application.Application.launch;

public class Launcher {

    static {
        SLF4JBridgeHandler.install();
    }

    public static void main(String[] args) {
        setLoggerFilePath();
        final Logger logger = LoggerFactory.getLogger(Launcher.class);
        try {


            // Loading lib secret keyring requires org.objectweb.asm, but including it clashes with lombok.
            // See: https://github.com/projectlombok/lombok/issues/2973
            // Workaround is to disable the use of asm library.
            System.setProperty("jnr.ffi.asm.enabled", "false");

            Thread.setDefaultUncaughtExceptionHandler((t, e) -> logger.error("Uncaught Exception: ", e));

            System.setProperty("javafx.preloader", PreloaderImpl.class.getCanonicalName());

            launch(FxApplication.class, args);
        } catch (Exception e) {
            logger.error("Uncaught Main Exception: ", e);
        }
    }

    private static void setLoggerFilePath() {
        // Set the path for file logging to user directory.
        // TODO  System.setProperty("correomqtt-logfile", SettingsProvider.getInstance().getLogPath());

        LoggerContext loggerContext = (LoggerContext) LoggerFactory.getILoggerFactory();
        loggerContext.reset();
        JoranConfigurator configurator = new JoranConfigurator();
        try (InputStream configStream = Launcher.class.getResourceAsStream("logback.xml")) {
            configurator.setContext(loggerContext);
            configurator.doConfigure(configStream);
        } catch (JoranException | IOException e) {
            System.out.println("Problem configuring logger: " + e.getMessage());
        }
    }
}
