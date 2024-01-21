package org.correomqtt.gui.views.settings;

import javafx.collections.FXCollections;
import javafx.fxml.FXML;
import javafx.scene.control.CheckBox;
import javafx.scene.control.ComboBox;
import javafx.scene.control.Label;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.VBox;
import javafx.stage.Stage;
import javafx.util.StringConverter;
import org.correomqtt.core.keyring.KeyringFactory;
import org.correomqtt.core.model.SettingsDTO;
import org.correomqtt.core.model.ThemeDTO;
import org.correomqtt.core.plugin.PluginManager;
import org.correomqtt.core.settings.SettingsProvider;
import org.correomqtt.gui.keyring.KeyringHandler;
import org.correomqtt.gui.model.KeyringModel;
import org.correomqtt.gui.model.LanguageModel;
import org.correomqtt.gui.model.WindowProperty;
import org.correomqtt.gui.model.WindowType;
import org.correomqtt.gui.plugin.spi.ThemeProviderHook;
import org.correomqtt.gui.theme.ThemeManager;
import org.correomqtt.gui.theme.ThemeProvider;
import org.correomqtt.gui.theme.light_legacy.LightLegacyThemeProvider;
import org.correomqtt.gui.utils.AlertHelper;
import org.correomqtt.gui.views.LoaderResult;
import org.correomqtt.gui.views.base.BaseControllerImpl;
import org.correomqtt.gui.views.cell.GenericCell;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import javax.inject.Inject;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.ResourceBundle;
import java.util.Set;
import java.util.concurrent.ConcurrentHashMap;
import java.util.function.Function;
import java.util.function.Predicate;
import java.util.stream.Collectors;

public class SettingsViewController extends BaseControllerImpl {

    private static final Logger LOGGER = LoggerFactory.getLogger(SettingsViewController.class);
    private final SettingsProvider settingsProvider;
    private final KeyringHandler keyringHandler;
    private final PluginManager pluginManager;
    private final KeyringFactory keyringFactory;
    private final AlertHelper alertHelper;
    private final GenericCell.Factory<ThemeProvider> themeProviderGenericCellFactory;
    private final GenericCell.Factory<KeyringModel> keyringModelGenericCellFactory;
    private final GenericCell.Factory<LanguageModel> languageModelGenericCellFactory;
    @FXML
    private AnchorPane settingsPane;
    @FXML
    private VBox settingsVBox;
    @FXML
    private ComboBox<ThemeProvider> themeComboBox;
    @FXML
    private ComboBox<LanguageModel> languageComboBox;
    @FXML
    private ComboBox<KeyringModel> keyringBackendComboBox;
    @FXML
    private CheckBox searchUpdatesCheckbox;
    @FXML
    private Label keyringDescriptionLabel;
    private ResourceBundle resources;
    private SettingsDTO settings;

    @Inject
    public SettingsViewController(SettingsProvider settingsProvider,
                                  ThemeManager themeManager,
                                  KeyringHandler keyringHandler,
                                  PluginManager pluginManager,
                                  KeyringFactory keyringFactory,
                                  AlertHelper alertHelper,
                                  GenericCell.Factory<ThemeProvider> themeProviderGenericCellFactory,
                                  GenericCell.Factory<KeyringModel> keyringModelGenericCellFactory,
                                  GenericCell.Factory<LanguageModel> languageModelGenericCellFactory) {
        super(settingsProvider, themeManager);
        this.settingsProvider = settingsProvider;
        this.keyringHandler = keyringHandler;
        this.pluginManager = pluginManager;
        this.keyringFactory = keyringFactory;
        resources = ResourceBundle.getBundle("org.correomqtt.i18n", settingsProvider.getSettings().getCurrentLocale());
        this.alertHelper = alertHelper;

        this.themeProviderGenericCellFactory = themeProviderGenericCellFactory;
        this.keyringModelGenericCellFactory = keyringModelGenericCellFactory;
        this.languageModelGenericCellFactory = languageModelGenericCellFactory;
    }

    public LoaderResult<SettingsViewController> showAsDialog() {
        LoaderResult<SettingsViewController> result = load();
        resources = result.getResourceBundle();

        Map<Object, Object> properties = new HashMap<>();
        properties.put(WindowProperty.WINDOW_TYPE, WindowType.SETTINGS);

        showAsDialog(result, resources.getString("settingsViewControllerTitle"), properties, false, false, null, event -> result.getController().keyHandling(event));
        return result;
    }

    public LoaderResult<SettingsViewController> load() {
        return load(SettingsViewController.class, "settingsView.fxml", () -> this);
    }

    private void keyHandling(KeyEvent event) {
        if (KeyCode.ESCAPE == event.getCode()) {
            closeDialog();
        }
    }

    private void closeDialog() {
        Stage stage = (Stage) themeComboBox.getScene().getWindow();
        stage.close();
    }

    @FXML
    private void initialize() {
        settings = settingsProvider.getSettings();
        setupGUI();

        // Unfortunately @FXML Handler does not work with Combobox, so the action must be bound manually.
        themeComboBox.setOnAction(actionEvent -> onThemeChanged());
        languageComboBox.setOnAction(actionEvent -> onLanguageChanged());
    }

    private void setupGUI() {
        searchUpdatesCheckbox.setSelected(settings.isSearchUpdates());

        ArrayList<ThemeProvider> themes = new ArrayList<>(pluginManager.getExtensions(ThemeProviderHook.class));
        if (LOGGER.isInfoEnabled()) {
            LOGGER.info(themes.stream().map(ThemeProvider::getName).collect(Collectors.joining(",")));
        }

        themeComboBox.setOnAction(null);
        themeComboBox.setItems(FXCollections.observableArrayList(themes));
        themeComboBox.setCellFactory(themeProviderGenericCellFactory::create);
        themeComboBox.setConverter(new StringConverter<>() {
            @Override
            public String toString(ThemeProvider object) {
                if (object == null) {
                    return null;
                }
                return object.getName();
            }

            @Override
            public ThemeProvider fromString(String string) {
                return null;
            }
        });

        themeComboBox.getSelectionModel().select(themes.
                stream()
                .filter(t -> {
                    if (settingsProvider.getThemeSettings().getNextTheme() != null) {
                        return t.getName().equals(settingsProvider.getThemeSettings().getNextTheme().getName());
                    }
                    return t.getName().equals(settingsProvider.getThemeSettings().getActiveTheme().getName());
                })
                .findFirst()
                .orElse(new LightLegacyThemeProvider()));

        List<KeyringModel> keyringModels = keyringFactory.getSupportedKeyrings()
                .stream()
                .map(KeyringModel::new)
                .toList();
        keyringBackendComboBox.setOnAction(event -> updateKeyringDescription(keyringBackendComboBox.getSelectionModel().getSelectedItem()));
        keyringBackendComboBox.setItems(FXCollections.observableArrayList(keyringModels));
        keyringBackendComboBox.setCellFactory(keyringModelGenericCellFactory::create);
        keyringBackendComboBox.setConverter(new StringConverter<>() {
            @Override
            public String toString(KeyringModel object) {
                if (object == null) {
                    return null;
                }
                return object.getLabelTranslationKey();
            }

            @Override
            public KeyringModel fromString(String string) {
                return null;
            }
        });

        KeyringModel selectedKeyring = keyringModels.
                stream()
                .filter(t -> t.getKeyring().getIdentifier().equals(settingsProvider.getSettings().getKeyringIdentifier()))
                .findFirst()
                .orElse(null);

        keyringBackendComboBox.getSelectionModel().select(selectedKeyring);

        if (selectedKeyring != null) {
            updateKeyringDescription(selectedKeyring);
        }

        languageComboBox.setCellFactory(languageModelGenericCellFactory::create);
        languageComboBox.setConverter(new StringConverter<>() {
            @Override
            public String toString(LanguageModel language) {
                return language.getLabelTranslationKey();
            }

            @Override
            public LanguageModel fromString(String string) {
                return null;
            }
        });

        Set<Locale> availableLocales = new HashSet<>();
        for (Locale locale : Locale.getAvailableLocales()) {
            if (SettingsViewController.class.getResource(
                    "/org/correomqtt/i18n_" + locale.getLanguage() + "_" + locale.getCountry() + ".properties") != null
            ) {
                availableLocales.add(locale);
            }
        }

        languageComboBox.setItems(FXCollections.observableArrayList(
                availableLocales.stream()
                        .filter(distinctByKey(l -> l.getLanguage() + "_" + l.getCountry()))
                        .map(LanguageModel::new)
                        .toList()));
        languageComboBox.getSelectionModel().select(new LanguageModel(settings.getSavedLocale()));

        settingsPane.setMinHeight(500);
        settingsPane.setMaxHeight(500);
    }

    @FXML
    private void onThemeChanged() {
        LOGGER.debug("Theme changed in settings");
    }

    private void onLanguageChanged() {
        // nothing to do
    }

    private void updateKeyringDescription(KeyringModel selectedKeyring) {
        keyringDescriptionLabel.setText(resources.getString("settingsViewKeyringBackendExplanationLabel")
                + "\n\n"
                + resources.getString(selectedKeyring.getLabelTranslationKey()) + ":\n"
                + resources.getString(selectedKeyring.getKeyring().getDescription()));
    }

    private static <T> Predicate<T> distinctByKey(Function<? super T, ?> keyExtractor) {
        Set<Object> seen = ConcurrentHashMap.newKeySet();
        return t -> seen.add(keyExtractor.apply(t));
    }

    @FXML
    private void onCancelClicked() {
        LOGGER.debug("Cancel in settings clicked");
        closeDialog();
    }

    @FXML
    private void onSaveClicked() {
        LOGGER.debug("Save in settings clicked");
        saveSettings();
        closeDialog();
    }

    private void saveSettings() {
        LOGGER.debug("Saving settings");

        String newKeyringIdentifier = keyringBackendComboBox.getSelectionModel().getSelectedItem().getKeyring().getIdentifier();
        String oldKeyringIdentifier = settings.getKeyringIdentifier();
        if (!newKeyringIdentifier.equals(oldKeyringIdentifier)) {
            settings.setKeyringIdentifier(newKeyringIdentifier);
            keyringHandler.migrate(newKeyringIdentifier);
        }
        settings.setSearchUpdates(searchUpdatesCheckbox.isSelected());
        ThemeProvider selectedTheme = themeComboBox.getSelectionModel().getSelectedItem();
        settings.setSavedLocale(languageComboBox.getSelectionModel().getSelectedItem().getLocale());
        settingsProvider.getThemeSettings().setNextTheme(
                new ThemeDTO(selectedTheme.getName()));
        settingsProvider.saveSettings();
        themeManager.saveCSS();
    }

    @FXML
    private void onWipeKeyringClicked() {
        boolean confirmed = alertHelper.confirm(
                resources.getString("wipeCurrentKeyringTitle"),
                resources.getString("wipeCurrentKeyringHeader"),
                resources.getString("wipeCurrentKeyringContent"),
                resources.getString("commonCancelButton"),
                resources.getString("wipeOutYesButton"));

        if (confirmed) {
            keyringHandler.retryWithMasterPassword(
                    masterPassword -> {
                        keyringHandler.wipe();
                        settingsProvider.wipeSecretData(masterPassword);
                    },
                    resources.getString("onPasswordWipeFailedTitle"),
                    resources.getString("onPasswordWipeFailedHeader"),
                    resources.getString("onPasswordWipeFailedContent"),
                    resources.getString("onPasswordWipeFailedGiveUp"),
                    resources.getString("onPasswordWipeFailedTryAgain")
            );
        }
    }

    @FXML
    private void onKeyringBackendChanged() {
        LOGGER.debug("Keyring backend changed in settings");
    }
}
