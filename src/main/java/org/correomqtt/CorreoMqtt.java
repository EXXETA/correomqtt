package org.correomqtt;

import ch.qos.logback.classic.LoggerContext;
import ch.qos.logback.classic.util.ContextInitializer;
import ch.qos.logback.core.joran.spi.JoranException;
import ch.qos.logback.core.util.StatusPrinter;
import com.exxeta.correomqtt.business.dispatcher.ApplicationLifecycleDispatcher;
import com.exxeta.correomqtt.business.dispatcher.ShortcutDispatcher;
import com.exxeta.correomqtt.business.model.SettingsDTO;
import com.exxeta.correomqtt.business.services.ConfigService;
import com.exxeta.correomqtt.business.utils.VersionUtils;
import com.exxeta.correomqtt.gui.controller.AlertController;
import com.exxeta.correomqtt.gui.controller.MainViewController;
import com.exxeta.correomqtt.gui.helper.AlertHelper;
import com.exxeta.correomqtt.gui.utils.CheckNewVersionUtils;
import com.exxeta.correomqtt.gui.utils.HostServicesHolder;
import com.exxeta.correomqtt.plugin.manager.PluginSystem;
import com.exxeta.correomqtt.plugin.update.PluginUpdateManager;
import javafx.application.Application;
import javafx.application.Platform;
import javafx.fxml.FXMLLoader;
import javafx.scene.Parent;
import javafx.scene.Scene;
import javafx.scene.control.Alert;
import javafx.scene.control.ButtonType;
import javafx.scene.control.DialogPane;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.stage.Stage;
import javafx.util.Pair;
import org.json.simple.parser.ParseException;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;
import java.net.MalformedURLException;
import java.util.Locale;
import java.util.Optional;
import java.util.ResourceBundle;

public class CorreoMqtt extends Application {

    private static final Logger LOGGER = LoggerFactory.getLogger(CorreoMqtt.class);
    private ResourceBundle resources;
    private MainViewController mainViewController;
    private Scene scene;

    public static void main(String[] args) {
        launch(args);
    }

    @Override
    public void start(Stage primaryStage) throws Exception {

        LOGGER.info("Application started.");
        LOGGER.info("JVM: {} | {} | {}.", System.getProperty("java.vendor"), System.getProperty("java.runtime.name"), System.getProperty("java.runtime.version"));
        LOGGER.info("CorreoMQTT version is {}.", VersionUtils.getVersion());

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

        resources = ResourceBundle.getBundle("com.exxeta.correomqtt.i18n", ConfigService.getInstance().getSettings().getCurrentLocale());

        LOGGER.info("Locale is: {}", settings.getSavedLocale());

        HostServicesHolder.getInstance().setHostServices(getHostServices());

        setLoggerFilePath();

        loadPrimaryStage(primaryStage);

        settings = ConfigService.getInstance().getSettings();

        if (settings.isFirstStart()) {
            boolean checkForUpdates = AlertHelper.confirm(
                    resources.getString("settingsViewUpdateLabel"),
                    resources.getString("firstStartCheckForUpdatesTitle"),
                    "",
                    resources.getString("commonNoButton"),
                    resources.getString("commonYesButton")
            );

            settings.setFirstStart(false);

            if (checkForUpdates) {
                settings.setSearchUpdates(true);
            } else {
                settings.setSearchUpdates(false);
            }
            ConfigService.getInstance().saveSettings();
        }

        settings = ConfigService.getInstance().getSettings();

        if (settings.isSearchUpdates()) {
            checkForUpdates();
        }

        AlertController.activate();
    }

    private void checkForUpdates() throws IOException, ParseException {
        PluginSystem pluginSystem = PluginSystem.getInstance();
        pluginSystem.loadPlugins();
        new PluginUpdateManager(pluginSystem).updateSystem();
        pluginSystem.startPlugins();

        CheckNewVersionUtils.checkNewVersion(false);
    }

    private void loadPrimaryStage(Stage primaryStage) throws IOException, ParseException {
        ConfigService.getInstance().setCssFileName();
        String cssPath = ConfigService.getInstance().getCssPath();

        FXMLLoader loader = new FXMLLoader(MainViewController.class.getResource("mainView.fxml"),
                                           ResourceBundle.getBundle("com.exxeta.correomqtt.i18n", ConfigService.getInstance().getSettings().getCurrentLocale()));
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
