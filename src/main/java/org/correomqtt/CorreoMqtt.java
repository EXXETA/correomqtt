package org.correomqtt;

import ch.qos.logback.classic.LoggerContext;
import ch.qos.logback.classic.util.ContextInitializer;
import ch.qos.logback.core.joran.spi.JoranException;
import ch.qos.logback.core.util.StatusPrinter;
import com.sun.javafx.application.LauncherImpl;
import javafx.application.Preloader;
import org.correomqtt.business.dispatcher.ApplicationLifecycleDispatcher;
import org.correomqtt.business.dispatcher.ShortcutDispatcher;
import org.correomqtt.business.model.SettingsDTO;
import org.correomqtt.business.services.ConfigService;
import org.correomqtt.business.utils.VersionUtils;
import org.correomqtt.gui.controller.AlertController;
import org.correomqtt.gui.controller.MainViewController;
import org.correomqtt.gui.helper.AlertHelper;
import org.correomqtt.gui.utils.CheckNewVersionUtils;
import org.correomqtt.gui.utils.HostServicesHolder;
import org.correomqtt.plugin.manager.PluginSystem;
import org.correomqtt.plugin.update.PluginUpdateManager;
import javafx.application.Application;
import javafx.application.Platform;
import javafx.fxml.FXMLLoader;
import javafx.scene.Parent;
import javafx.scene.Scene;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.stage.Stage;
import org.json.simple.parser.ParseException;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;
import java.util.Locale;
import java.util.ResourceBundle;
import java.util.concurrent.Callable;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.FutureTask;

public class CorreoMqtt extends Application {

    private static final Logger LOGGER = LoggerFactory.getLogger(CorreoMqtt.class);
    private ResourceBundle resources;
    private MainViewController mainViewController;
    private Scene scene;

    public static void main(String[] args) {
        launch(args);
    }

    @Override
    public void init() throws IOException, ParseException, InterruptedException {
        SettingsDTO settings = ConfigService.getInstance().getSettings();

        if (settings.getSavedLocale() == null) {
            if (Locale.getDefault().getLanguage().equals("de") &&
                    Locale.getDefault().getCountry().equals("DE")) {
                settings.setSavedLocale(new Locale("de", "DE"));
            } else {
                settings.setSavedLocale(new Locale("en", "US"));
            }
        }

        settings.setCurrentLocale(settings.getSavedLocale());
        ConfigService.getInstance().saveSettings();

        LauncherImpl.notifyPreloader(this, new Preloader.ProgressNotification(0));

        resources = ResourceBundle.getBundle("org.correomqtt.i18n", ConfigService.getInstance().getSettings().getCurrentLocale());

        LOGGER.info("Locale is: {}", settings.getSavedLocale());

        HostServicesHolder.getInstance().setHostServices(getHostServices());

        setLoggerFilePath();

        SettingsDTO settings1 = ConfigService.getInstance().getSettings();

        if (settings1.isFirstStart()) {

            CountDownLatch countDownLatch = new CountDownLatch(1);
            Platform.runLater(() -> {
                boolean checkForUpdates = AlertHelper.confirm(
                        resources.getString("settingsViewUpdateLabel"),
                        null,
                        resources.getString("firstStartCheckForUpdatesTitle"),
                        resources.getString("commonNoButton"),
                        resources.getString("commonYesButton")
                );

                settings1.setFirstStart(false);

                if (checkForUpdates) {
                    settings1.setSearchUpdates(true);
                } else {
                    settings1.setSearchUpdates(false);
                }
                ConfigService.getInstance().saveSettings();
                countDownLatch.countDown();
            });

            countDownLatch.await();
        }

        if (settings.isSearchUpdates()) {
            LauncherImpl.notifyPreloader(this, new Preloader.ProgressNotification(10));
            checkForUpdates();
        }

        LauncherImpl.notifyPreloader(this, new Preloader.ProgressNotification(20));
    }

    private void checkForUpdates() throws IOException, InterruptedException {
        PluginSystem pluginSystem = PluginSystem.getInstance();
        pluginSystem.loadPlugins();
        new PluginUpdateManager(pluginSystem).updateSystem();
        pluginSystem.startPlugins();

        CountDownLatch countDownLatch = new CountDownLatch(1);
        Platform.runLater(() -> {
            try {
                CheckNewVersionUtils.checkNewVersion(false, countDownLatch);
            } catch (IOException e) {
                e.printStackTrace();
            } catch (ParseException e) {
                e.printStackTrace();
            }
        });
        countDownLatch.await();
    }

    @Override
    public void start(Stage primaryStage) throws Exception {

        LOGGER.info("Application started.");
        LOGGER.info("JVM: {} | {} | {}.", System.getProperty("java.vendor"), System.getProperty("java.runtime.name"), System.getProperty("java.runtime.version"));
        LOGGER.info("CorreoMQTT version is {}.", VersionUtils.getVersion());

        loadPrimaryStage(primaryStage);

        AlertController.activate();
    }

    private void loadPrimaryStage(Stage primaryStage) throws IOException {
        ConfigService.getInstance().setCssFileName();
        String cssPath = ConfigService.getInstance().getCssPath();

        FXMLLoader loader = new FXMLLoader(MainViewController.class.getResource("mainView.fxml"),
                ResourceBundle.getBundle("org.correomqtt.i18n", ConfigService.getInstance().getSettings().getCurrentLocale()));
        Parent root = loader.load();

        mainViewController = loader.getController();
        primaryStage.setTitle("CorreoMQTT v" + VersionUtils.getVersion());
        scene = new Scene(root, 900, 800);

        if (cssPath != null) {
            scene.getStylesheets().add(cssPath);
        }
        primaryStage.setScene(scene);
        primaryStage.setMinHeight(400);
        primaryStage.setMinWidth(850);
        primaryStage.show();

        primaryStage.setOnCloseRequest(t -> {
            LOGGER.info("Main window closed. Initialize shutdown.");
            LOGGER.info("Shutting down connections.");
            ApplicationLifecycleDispatcher.getInstance().onShutdown();
            LOGGER.info("Shutting down plugins.");
            PluginSystem.getInstance().stopPlugins();
            LOGGER.info("Shutting down application. Bye.");
            Platform.exit();
            System.exit(0);
        });

        setupShortcut();
    }

    private void setupShortcut() {
        scene.addEventHandler(KeyEvent.KEY_PRESSED, event -> {

                    if (event.getCode().equals(KeyCode.S) && event.isShortcutDown() && !event.isShiftDown()) {
                        ShortcutDispatcher.getInstance().onSubscriptionShortcutPressed(mainViewController.getUUIDofSelectedTab());
                        event.consume();
                    }
                    if (event.getCode().equals(KeyCode.S) && event.isShortcutDown() && event.isShiftDown()) {
                        ShortcutDispatcher.getInstance().onClearIncomingShortcutPressed(mainViewController.getUUIDofSelectedTab());
                        event.consume();
                    }
                    if (event.getCode().equals(KeyCode.P) && event.isShortcutDown() && !event.isShiftDown()) {
                        ShortcutDispatcher.getInstance().onPublishShortcutPressed(mainViewController.getUUIDofSelectedTab());
                        event.consume();
                    }
                    if (event.getCode().equals(KeyCode.P) && event.isShortcutDown() && event.isShiftDown()) {
                        ShortcutDispatcher.getInstance().onClearOutgoingShortcutPressed(mainViewController.getUUIDofSelectedTab());
                        event.consume();
                    }
                    //TODO rest
                }
        );
    }

    private void setLoggerFilePath() {

        // Set the path for file logging to user directory.
        System.setProperty("correomqtt-logfile", ConfigService.getInstance().getLogPath());

        LoggerContext lc = (LoggerContext) LoggerFactory.getILoggerFactory();
        ContextInitializer ci = new ContextInitializer(lc);
        lc.reset();
        try {
            //I prefer autoConfig() over JoranConfigurator.doConfigure() so I wouldn't need to find the file myself.
            ci.autoConfig();
        } catch (JoranException e) {
            // StatusPrinter will try to log this
            e.printStackTrace(); //TODO
        }
        StatusPrinter.printInCaseOfErrorsOrWarnings(lc);
    }

}
