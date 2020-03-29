package org.correomqtt.gui.controller;

import org.correomqtt.business.model.SettingsDTO;
import org.correomqtt.business.model.ThemeDTO;
import org.correomqtt.business.model.ThemeSettingsDTO;
import org.correomqtt.business.services.ConfigService;
import org.correomqtt.gui.model.WindowProperty;
import org.correomqtt.gui.model.WindowType;
import javafx.collections.FXCollections;
import javafx.fxml.FXML;
import javafx.scene.control.CheckBox;
import javafx.scene.control.ComboBox;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.stage.Stage;
import javafx.util.StringConverter;
import org.apache.commons.lang3.StringUtils;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.*;
import java.util.concurrent.ConcurrentHashMap;
import java.util.function.Function;
import java.util.function.Predicate;
import java.util.stream.Collectors;

public class SettingsViewController extends BaseController {

    private static ResourceBundle resources;

    @FXML
    private ComboBox<ThemeDTO> themeComboBox;
    @FXML
    private ComboBox<Locale> languageComboBox;
    @FXML
    private CheckBox searchUpdatesCheckbox;

    private SettingsDTO settings;
    private ThemeSettingsDTO themeSettings;
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
        settings = ConfigService.getInstance().getSettings();
        themeSettings = ConfigService.getInstance().getThemeSettings();
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
    public void onThemeChanged() {
        LOGGER.debug("Theme changed in settings");
    }

    private void setupGUI() {
        searchUpdatesCheckbox.setSelected(settings.isSearchUpdates());

        List<ThemeDTO> themes = new ArrayList<>();
        themeSettings.getThemes().forEach(t -> themes.add(t));

        themeComboBox.setOnAction(null);
        themeComboBox.setItems(FXCollections.observableArrayList(themes));
        themeComboBox.setConverter(new StringConverter<ThemeDTO>() {
            @Override
            public String toString(ThemeDTO object) {
                if (object == null) {
                    return null;
                }
                return object.getName();
            }

            @Override
            public ThemeDTO fromString(String string) {
                return null;
            }
        });
        themeComboBox.getSelectionModel().select(themeSettings.getActiveTheme());

        languageComboBox.setConverter(new StringConverter<>() {
            @Override
            public String toString(Locale object) {
                return StringUtils.capitalize(object.getDisplayLanguage(object));
            }

            @Override
            public Locale fromString(String string) {
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
                        .collect(Collectors.toList())));
        languageComboBox.getSelectionModel().select(settings.getSavedLocale());
    }

    private static <T> Predicate<T> distinctByKey(Function<? super T, ?> keyExtractor) {
        Set<Object> seen = ConcurrentHashMap.newKeySet();
        return t -> seen.add(keyExtractor.apply(t));
    }

    private void saveSettings() {
        LOGGER.debug("Saving settings");

        settings.setSearchUpdates(searchUpdatesCheckbox.isSelected());
        ThemeDTO selectedTheme = themeComboBox.getSelectionModel().getSelectedItem();
        settings.setSavedLocale(languageComboBox.getSelectionModel().getSelectedItem());
        ConfigService.getInstance().saveSettings();
        themeSettings.setActiveTheme(selectedTheme);
        ConfigService.getInstance().saveThemeSettings();
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
