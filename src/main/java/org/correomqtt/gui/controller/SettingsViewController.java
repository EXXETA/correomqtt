package org.correomqtt.gui.controller;

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
import org.correomqtt.plugin.spi.ThemeProviderHook;
import org.correomqtt.business.keyring.KeyringFactory;
import org.correomqtt.business.model.SettingsDTO;
import org.correomqtt.business.model.ThemeDTO;
import org.correomqtt.business.provider.SettingsProvider;
import org.correomqtt.gui.cell.GenericCell;
import org.correomqtt.gui.helper.AlertHelper;
import org.correomqtt.gui.keyring.KeyringHandler;
import org.correomqtt.gui.model.KeyringModel;
import org.correomqtt.gui.model.LanguageModel;
import org.correomqtt.gui.model.WindowProperty;
import org.correomqtt.gui.model.WindowType;
import org.correomqtt.gui.theme.ThemeProvider;
import org.correomqtt.gui.theme.light_legacy.LightLegacyThemeProvider;
import org.correomqtt.plugin.manager.PluginManager;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

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

    private static ResourceBundle resources = ResourceBundle.getBundle("org.correomqtt.i18n", SettingsProvider.getInstance().getSettings().getCurrentLocale());

    private SettingsDTO settings;
    private static final Logger LOGGER = LoggerFactory.getLogger(SettingsViewController.class);

    public static LoaderResult<SettingsViewController> load() {
        return load(SettingsViewController.class, "settingsView.fxml");
    }

    public static LoaderResult<SettingsViewController> showAsDialog() {
        LoaderResult<SettingsViewController> result = load();
        resources = result.getResourceBundle();

        Map<Object, Object> properties = new HashMap<>();
        properties.put(WindowProperty.WINDOW_TYPE, WindowType.SETTINGS);

        showAsDialog(result, resources.getString("settingsViewControllerTitle"), properties, false, false, null, event -> result.getController().keyHandling(event));
        return result;
    }

    @FXML
    public void initialize() {
        settings = SettingsProvider.getInstance().getSettings();
        setupGUI();

        // Unfortunately @FXML Handler does not work with Combobox, so the action must be bound manually.
        themeComboBox.setOnAction(actionEvent -> onThemeChanged());
        languageComboBox.setOnAction(actionEvent -> onLanguageChanged());
    }

    private void onLanguageChanged() {
        // nothing to do
    }

    @FXML
    public void onCancelClicked() {
        LOGGER.debug("Cancel in settings clicked");
        closeDialog();
    }

    @FXML
    public void onSaveClicked() {
        LOGGER.debug("Save in settings clicked");
        saveSettings();
        closeDialog();
    }

    @FXML
    public void onWipeKeyringClicked() {
        boolean confirmed = AlertHelper.confirm(
                resources.getString("wipeCurrentKeyringTitle"),
                resources.getString("wipeCurrentKeyringHeader"),
                resources.getString("wipeCurrentKeyringContent"),
                resources.getString("commonCancelButton"),
                resources.getString("wipeOutYesButton"));

        if (confirmed) {
            KeyringHandler.getInstance().retryWithMasterPassword(
                    masterPassword -> SettingsProvider.getInstance().wipeSecretData(masterPassword),
                    resources.getString("onPasswordWipeFailedTitle"),
                    resources.getString("onPasswordWipeFailedHeader"),
                    resources.getString("onPasswordWipeFailedContent"),
                    resources.getString("onPasswordWipeFailedGiveUp"),
                    resources.getString("onPasswordWipeFailedTryAgain")
            );
        }
    }

    @FXML
    public void onThemeChanged() {
        LOGGER.debug("Theme changed in settings");
    }

    @FXML
    public void onKeyringBackendChanged() {
        LOGGER.debug("Keyring backend changed in settings");
    }

    private void setupGUI() {
        searchUpdatesCheckbox.setSelected(settings.isSearchUpdates());

        ArrayList<ThemeProvider> themes = new ArrayList<>(PluginManager.getInstance().getExtensions(ThemeProviderHook.class));
        if(LOGGER.isInfoEnabled()) {
            LOGGER.info(themes.stream().map(ThemeProvider::getName).collect(Collectors.joining(",")));
        }

        themeComboBox.setOnAction(null);
        themeComboBox.setItems(FXCollections.observableArrayList(themes));
        themeComboBox.setCellFactory(GenericCell::new);
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
                    if(SettingsProvider.getInstance().getThemeSettings().getNextTheme() != null) {
                        return t.getName().equals(SettingsProvider.getInstance().getThemeSettings().getNextTheme().getName());
                    }
                    return t.getName().equals(SettingsProvider.getInstance().getThemeSettings().getActiveTheme().getName());
                })
                .findFirst()
                .orElse(new LightLegacyThemeProvider()));

        List<KeyringModel> keyringModels = KeyringFactory.getSupportedKeyrings()
                .stream()
                .map(KeyringModel::new)
                .toList();
        keyringBackendComboBox.setOnAction(event -> updateKeyringDescription(keyringBackendComboBox.getSelectionModel().getSelectedItem()));
        keyringBackendComboBox.setItems(FXCollections.observableArrayList(keyringModels));
        keyringBackendComboBox.setCellFactory(GenericCell::new);
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
                .filter(t -> t.getKeyring().getIdentifier().equals(SettingsProvider.getInstance().getSettings().getKeyringIdentifier()))
                .findFirst()
                .orElse(null);

        keyringBackendComboBox.getSelectionModel().select(selectedKeyring);

        if (selectedKeyring != null) {
            updateKeyringDescription(selectedKeyring);
        }

        languageComboBox.setCellFactory(GenericCell::new);
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

    private void updateKeyringDescription(KeyringModel selectedKeyring) {
        keyringDescriptionLabel.setText(resources.getString("settingsViewKeyringBackendExplanationLabel")
                + "\n\n"
                + selectedKeyring.getLabelTranslationKey() + ":\n"
                + selectedKeyring.getKeyring().getDescription());
    }

    private static <T> Predicate<T> distinctByKey(Function<? super T, ?> keyExtractor) {
        Set<Object> seen = ConcurrentHashMap.newKeySet();
        return t -> seen.add(keyExtractor.apply(t));
    }

    private void saveSettings() {
        LOGGER.debug("Saving settings");

        String newKeyringIdentifier = keyringBackendComboBox.getSelectionModel().getSelectedItem().getKeyring().getIdentifier();
        String oldKeyringIdentifier = settings.getKeyringIdentifier();
        if (!newKeyringIdentifier.equals(oldKeyringIdentifier)) {
            settings.setKeyringIdentifier(newKeyringIdentifier);
            KeyringHandler.getInstance().migrate(newKeyringIdentifier);
        }
        settings.setSearchUpdates(searchUpdatesCheckbox.isSelected());
        ThemeProvider selectedTheme = themeComboBox.getSelectionModel().getSelectedItem();
        settings.setSavedLocale(languageComboBox.getSelectionModel().getSelectedItem().getLocale());
        SettingsProvider.getInstance().getThemeSettings().setNextTheme(new ThemeDTO(selectedTheme.getName(), selectedTheme.getIconMode()));
        SettingsProvider.getInstance().saveSettings(true);
    }

    private void closeDialog() {
        Stage stage = (Stage) themeComboBox.getScene().getWindow();
        stage.close();
    }

    private void keyHandling(KeyEvent event) {
        if (KeyCode.ESCAPE == event.getCode()) {
            closeDialog();
        }
    }
}
