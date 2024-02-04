package org.correomqtt.gui.theme;

import org.correomqtt.core.settings.SettingsManager;
import org.correomqtt.core.eventbus.EventBus;
import org.correomqtt.core.fileprovider.BaseUserFileProvider;
import org.correomqtt.core.fileprovider.SettingsUpdatedEvent;
import org.correomqtt.core.plugin.PluginManager;
import org.correomqtt.gui.plugin.spi.ThemeProviderHook;
import org.correomqtt.gui.theme.light_legacy.LightLegacyThemeProvider;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import org.correomqtt.di.Inject;
import org.correomqtt.di.SingletonBean;
import java.io.File;
import java.util.ArrayList;

@SingletonBean
public class ThemeManager extends BaseUserFileProvider {

    private static final Logger LOGGER = LoggerFactory.getLogger(ThemeManager.class);
    private final SettingsManager settingsManager;
    private final PluginManager pluginManager;
    private ThemeProvider activeThemeProvider;

    private static final String CSS_FILE_NAME = "style.css";

    @Inject
    public ThemeManager(SettingsManager settingsManager,
                        EventBus eventBus,
                        PluginManager pluginManager) {
        super(eventBus);
        this.settingsManager = settingsManager;
        this.pluginManager = pluginManager;
    }

    public ThemeProvider getActiveTheme() {
        if (activeThemeProvider == null) {
            String activeThemeName = settingsManager.getActiveTheme();
            ArrayList<ThemeProvider> themes = new ArrayList<>(pluginManager.getExtensions(ThemeProviderHook.class));
            activeThemeProvider = themes.stream()
                    .filter(t -> t.getName().equals(activeThemeName))
                    .findFirst()
                    .orElse(new LightLegacyThemeProvider());
        }
        return activeThemeProvider;
    }


    public String getCssPath() {
        File cssFile = new File(getTargetDirectoryPath() + File.separator + CSS_FILE_NAME);
        if (!cssFile.exists()) {
            saveToUserDirectory(CSS_FILE_NAME, getActiveTheme().getCss());
            LOGGER.info("Write CSS to {}.", CSS_FILE_NAME);
        }
        if (cssFile.exists()) {
            return cssFile.toURI().toString();
        } else {
            return null;
        }
    }

    public String getIconModeCssClass() {
        return getActiveTheme().getIconMode().toString();
    }

    public void saveCSS() {
        saveToUserDirectory(CSS_FILE_NAME, getActiveTheme().getCss());
        LOGGER.info("Write CSS to {}.", CSS_FILE_NAME);
        eventBus.fire(new SettingsUpdatedEvent(false));
    }
}
