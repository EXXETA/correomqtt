package org.correomqtt;

import javafx.application.Application;
import javafx.application.Platform;
import javafx.application.Preloader;
import javafx.fxml.FXMLLoader;
import javafx.scene.Parent;
import javafx.scene.Scene;
import javafx.scene.control.ButtonBar;
import javafx.scene.control.ButtonType;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.stage.Stage;
import org.apache.maven.artifact.versioning.ComparableVersion;
import org.correomqtt.core.CorreoCore;
import org.correomqtt.core.applifecycle.ShutdownEvent;
import org.correomqtt.core.applifecycle.ShutdownRequestEvent;
import org.correomqtt.core.eventbus.EventBus;
import org.correomqtt.core.eventbus.Subscribe;
import org.correomqtt.core.exception.CorreoMqttUnableToCheckVersionException;
import org.correomqtt.core.model.GlobalUISettings;
import org.correomqtt.core.model.SettingsDTO;
import org.correomqtt.core.plugin.PluginManager;
import org.correomqtt.core.settings.SettingsManager;
import org.correomqtt.core.shortcut.ShortcutConnectionIdEvent;
import org.correomqtt.core.utils.VersionUtils;
import org.correomqtt.gui.keyring.KeyringManager;
import org.correomqtt.gui.plugin.CorreoExtensionFactory;
import org.correomqtt.gui.plugin.PluginLauncher;
import org.correomqtt.gui.theme.ThemeManager;
import org.correomqtt.gui.utils.AlertHelper;
import org.correomqtt.gui.utils.CheckNewVersionUtils;
import org.correomqtt.gui.utils.PluginCheckUtils;
import org.correomqtt.gui.views.AlertController;
import org.correomqtt.gui.views.MainViewController;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import javax.inject.Inject;
import javax.inject.Singleton;
import java.awt.*;
import java.io.IOException;
import java.util.Locale;
import java.util.ResourceBundle;
import java.util.function.Consumer;

import static org.correomqtt.core.shortcut.ShortcutEvent.Shortcut.CLEAR_INCOMING;
import static org.correomqtt.core.shortcut.ShortcutEvent.Shortcut.CLEAR_OUTGOING;
import static org.correomqtt.core.shortcut.ShortcutEvent.Shortcut.PUBLISH;
import static org.correomqtt.core.shortcut.ShortcutEvent.Shortcut.SUBSCRIPTION;

@Singleton
public class CorreoApp extends Application {

    private static final Logger LOGGER = LoggerFactory.getLogger(CorreoApp.class);
    private final PluginLauncher pluginLauncher;
    private final PluginManager pluginManager;
    private final KeyringManager keyringManager;
    private final SettingsManager settingsManager;
    private final ThemeManager themeManager;
    private final AlertHelper alertHelper;
    private final CheckNewVersionUtils checkNewVersionUtils;
    private final AlertController alertController;
    private final PluginCheckUtils pluginCheckUtils;
    private final MainViewController mainViewController;
    private final CorreoCore correoCore;
    private final EventBus eventBus;
    private ResourceBundle resources;
    private Scene scene;
    private Stage primaryStage;
    private Consumer<Preloader.PreloaderNotification> notifyPreloader;

    @Inject
    public CorreoApp(PluginManager pluginManager,
                     PluginLauncher pluginLauncher,
                     KeyringManager keyringManager,
                     SettingsManager settingsManager,
                     ThemeManager themeManager,
                     AlertHelper alertHelper,
                     CheckNewVersionUtils checkNewVersionUtils,
                     AlertController alertController,
                     PluginCheckUtils pluginCheckUtils,
                     MainViewController mainViewController,
                     CorreoCore correoCore,
                     EventBus eventBus) {
        this.pluginManager = pluginManager;
        this.pluginLauncher = pluginLauncher;
        this.keyringManager = keyringManager;
        this.settingsManager = settingsManager;
        this.themeManager = themeManager;
        this.alertHelper = alertHelper;
        this.checkNewVersionUtils = checkNewVersionUtils;
        this.alertController = alertController;
        this.pluginCheckUtils = pluginCheckUtils;
        this.mainViewController = mainViewController;
        this.correoCore = correoCore;
        this.eventBus = eventBus;
    }

    @Override
    public void start(Stage primaryStage) throws IOException {
        this.primaryStage = primaryStage;
        loadPrimaryStage();

        alertController.activate();
    }

    @Override
    public void init() {

        alertController.activate();
        eventBus.register(this);

        if (LOGGER.isInfoEnabled()) {
            LOGGER.info("Application started.");
            LOGGER.info("JVM: {} {} {}", System.getProperty("java.vendor"), System.getProperty("java.runtime.name"), System.getProperty("java.runtime.version"));
            LOGGER.info("JavaFX: {}, Runtime: {}", System.getProperty("javafx.version"), System.getProperty("javafx.runtime.version"));
            LOGGER.info("OS: {} {} {}", System.getProperty("os.name"), System.getProperty("os.version"), System.getProperty("os.arch"));
            Dimension screenSize = Toolkit.getDefaultToolkit().getScreenSize();
            String xdgCurrentDesktop = System.getenv("XDG_CURRENT_DESKTOP");
            LOGGER.info("ENV: {}{} x {} ", xdgCurrentDesktop != null ? xdgCurrentDesktop + " " : "", screenSize.getWidth(), screenSize.getHeight());
            LOGGER.info("CorreoMQTT version is {}", VersionUtils.getVersion());
        }

        final SettingsDTO settings = settingsManager.getSettings();

        handleVersionMismatch(settings);
        setLocale(settings);
        notifyPreloader.accept(new CorreoPreloaderNotification(resources.getString("preloaderLanguageSet")));

        if (settings.isFirstStart()) {
            initUpdatesOnFirstStart(settings);
        }

        pluginManager.setExtensionFactory(new CorreoExtensionFactory());
        pluginCheckUtils.checkMigration();

        if (settings.isSearchUpdates()) {
            notifyPreloader.accept(new CorreoPreloaderNotification(resources.getString("preloaderSearchingUpdates")));
            pluginLauncher.start(true);
            try {
                checkForUpdates();
            } catch (CorreoMqttUnableToCheckVersionException e) {
                LOGGER.debug("Unable to check version", e);
            }
        }

        notifyPreloader.accept(new CorreoPreloaderNotification(resources.getString("preloaderKeyring")));
        keyringManager.init();
        keyringManager.retryWithMasterPassword(
                settingsManager::initializePasswords,
                resources.getString("onPasswordReadFailedTitle"),
                resources.getString("onPasswordReadFailedHeader"),
                resources.getString("onPasswordReadFailedContent"),
                resources.getString("onPasswordReadFailedGiveUp"),
                resources.getString("onPasswordReadFailedTryAgain")
        );

        notifyPreloader.accept(new CorreoPreloaderNotification(resources.getString("preloaderReady")));
        settingsManager.saveSettings();

        System.setProperty("correo.iconModeCssClass", themeManager.getIconModeCssClass());
        correoCore.init();
    }

    private void loadPrimaryStage() throws IOException {
        String cssPath = themeManager.getCssPath();

        FXMLLoader loader = new FXMLLoader(MainViewController.class.getResource("mainView.fxml"),
                ResourceBundle.getBundle("org.correomqtt.i18n", settingsManager.getSettings().getCurrentLocale()));
        loader.setControllerFactory(param -> mainViewController);
        Parent root = loader.load();

        primaryStage.setTitle("CorreoMQTT v" + VersionUtils.getVersion());
        scene = new Scene(root, 900, 800);
        scene.setFill(themeManager.getActiveTheme().getBackgroundColor());

        if (cssPath != null) {
            scene.getStylesheets().add(cssPath);
        }
        primaryStage.setScene(scene);
        primaryStage.sizeToScene();
        primaryStage.setMinHeight(400);
        primaryStage.setMinWidth(850);

        final SettingsDTO settings = settingsManager.getSettings();

        if (settings.getGlobalUISettings() == null) {
            primaryStage.show();
            saveGlobalUISettings();
        } else {
            primaryStage.setX(settings.getGlobalUISettings().getWindowPositionX());
            primaryStage.setY(settings.getGlobalUISettings().getWindowPositionY());
            primaryStage.setWidth(settings.getGlobalUISettings().getWindowWidth());
            primaryStage.setHeight(settings.getGlobalUISettings().getWindowHeight());

            primaryStage.show();
        }

        primaryStage.setOnCloseRequest(t -> onShutdownRequested());

        setupShortcut();
    }

    private void handleVersionMismatch(SettingsDTO settings) {
        if (settings.getConfigCreatedWithCorreoVersion() == null) {
            LOGGER.info("Setting initial correo version in settings: {}", VersionUtils.getVersion());
            settings.setConfigCreatedWithCorreoVersion(VersionUtils.getVersion());
        } else if (new ComparableVersion(VersionUtils.getVersion())
                .compareTo(new ComparableVersion(settings.getConfigCreatedWithCorreoVersion())) > 0) {
            LOGGER.info("Installed version is newer than version which created the config file");
            // handle issues if new version needs some changes
        }
    }

    private void setLocale(SettingsDTO settings) {
        if (settings.getSavedLocale() == null) {
            if (Locale.getDefault().getLanguage().equals("de") &&
                    Locale.getDefault().getCountry().equals("DE")) {
                settings.setSavedLocale(new Locale("de", "DE"));
            } else {
                settings.setSavedLocale(new Locale("en", "US"));
            }
        }
        settings.setCurrentLocale(settings.getSavedLocale());
        LOGGER.info("Locale is: {}", settings.getSavedLocale());
        resources = ResourceBundle.getBundle("org.correomqtt.i18n", settingsManager.getSettings().getCurrentLocale());
    }

    private void initUpdatesOnFirstStart(SettingsDTO settings) {
        boolean checkForUpdates = alertHelper.confirm(
                resources.getString("settingsViewUpdateLabel"),
                null,
                resources.getString("firstStartCheckForUpdatesTitle"),
                resources.getString("commonNoButton"),
                resources.getString("commonYesButton")
        );

        settings.setFirstStart(false);
        settings.setSearchUpdates(checkForUpdates);
    }

    private boolean checkForUpdates() throws CorreoMqttUnableToCheckVersionException {
        try {
            checkNewVersionUtils.checkNewVersion(false);
            return true;
        } catch (IOException e) {
            LOGGER.warn("Version check failed.", e);
            return false;
        }
    }

    private void saveGlobalUISettings() {
        final SettingsDTO settings = settingsManager.getSettings();

        settings.setGlobalUISettings(new GlobalUISettings(
                primaryStage.getX(),
                primaryStage.getY(),
                primaryStage.getWidth(),
                primaryStage.getHeight()
        ));

        settingsManager.saveSettings();
    }

    @Subscribe(ShutdownRequestEvent.class)
    public void onShutdownRequested() {
        LOGGER.info("Main window closed. Initialize shutdown.");
        LOGGER.info("Saving global UI settings.");
        saveGlobalUISettings();
        LOGGER.info("Saving connection UI settings.");
        saveConnectionUISettings();
        LOGGER.info("Shutting down connections.");
        eventBus.fire(new ShutdownEvent());
        LOGGER.info("Shutting down plugins.");
        pluginManager.stopPlugins();
        LOGGER.info("Shutting down application. Bye.");
        alertController.deactivate();
        Platform.exit();
        System.exit(0);
    }

    private void setupShortcut() {
        scene.addEventHandler(KeyEvent.KEY_PRESSED, event -> {

                    if (event.getCode().equals(KeyCode.S) && event.isShortcutDown() && !event.isShiftDown()) {
                        eventBus.fireAsync(new ShortcutConnectionIdEvent(SUBSCRIPTION, mainViewController.getUUIDofSelectedTab()));
                        event.consume();
                    }
                    if (event.getCode().equals(KeyCode.S) && event.isShortcutDown() && event.isShiftDown()) {
                        eventBus.fireAsync(new ShortcutConnectionIdEvent(CLEAR_INCOMING, mainViewController.getUUIDofSelectedTab()));
                        event.consume();
                    }
                    if (event.getCode().equals(KeyCode.P) && event.isShortcutDown() && !event.isShiftDown()) {
                        eventBus.fireAsync(new ShortcutConnectionIdEvent(PUBLISH, mainViewController.getUUIDofSelectedTab()));
                        event.consume();
                    }
                    if (event.getCode().equals(KeyCode.P) && event.isShortcutDown() && event.isShiftDown()) {
                        eventBus.fireAsync(new ShortcutConnectionIdEvent(CLEAR_OUTGOING, mainViewController.getUUIDofSelectedTab()));
                        event.consume();
                    }
                    //TODO rest
                }
        );
    }

    private void saveConnectionUISettings() {
        mainViewController.getTabPane().getTabs().forEach(tab -> {
            if (mainViewController.getConnectionViewControllers().get(tab.getId()) != null) {
                mainViewController.getConnectionViewControllers().get(tab.getId()).saveConnectionUISettings();
            }
        });
    }

    //TODO use
    public void onPluginUpdateFailed(String disabledPath) {
        alertHelper.warn(
                resources.getString("pluginUpdateErrorTitle"),
                resources.getString("pluginUpdateErrorContent") + " " + disabledPath,
                true
        );
    }

    // TODO use
    public void onPluginLoadFailed() {
        alertHelper.warn(
                resources.getString("pluginErrorTitle"),
                resources.getString("pluginErrorContent"),
                true,
                new ButtonType(resources.getString("closeNow"), ButtonBar.ButtonData.OK_DONE)
        );
        System.exit(1);
    }

    @Override
    public void stop() {
        //TODO Do we need to do something on stop?
    }

    void onNotifyPreloader(Consumer<Preloader.PreloaderNotification> notifyPreloader) {
        this.notifyPreloader = notifyPreloader;
        pluginLauncher.onNotifyPreloader(notifyPreloader);
    }
}
