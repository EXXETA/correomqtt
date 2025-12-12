package org.correomqtt.gui.theme;

import org.correomqtt.core.fileprovider.BaseUserFileProvider;
import org.correomqtt.core.fileprovider.SettingsUpdatedEvent;
import org.correomqtt.core.plugin.PluginManager;
import org.correomqtt.core.settings.SettingsManager;
import org.correomqtt.core.utils.DirectoryUtils;
import org.correomqtt.di.Inject;
import org.correomqtt.di.SingletonBean;
import org.correomqtt.di.SoyEvents;
import org.correomqtt.gui.plugin.spi.ThemeProviderHook;
import org.correomqtt.gui.theme.light_legacy.LightLegacyThemeProvider;
import org.correomqtt.gui.theme.light_macos.LightMacosThemeProvider;
import org.correomqtt.gui.theme.light_win11.LightWin11ThemeProvider;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.BufferedReader;
import java.io.File;
import java.io.IOException;
import java.io.InputStreamReader;
import java.util.ArrayList;

import static com.sun.javafx.PlatformUtil.isLinux;
import static com.sun.javafx.PlatformUtil.isMac;
import static com.sun.javafx.PlatformUtil.isWindows;

@SingletonBean
public class ThemeManager extends BaseUserFileProvider {

    private static final Logger LOGGER = LoggerFactory.getLogger(ThemeManager.class);

    private final SettingsManager settingsManager;

    private final PluginManager pluginManager;

    private ThemeProvider activeThemeProvider;

    private static final String CSS_FILE_NAME = "style.css";

    @Inject
    public ThemeManager(SettingsManager settingsManager,
            SoyEvents soyEvents,
            PluginManager pluginManager) {
        super(soyEvents);
        this.settingsManager = settingsManager;
        this.pluginManager = pluginManager;
    }

    public ThemeProvider getActiveTheme() {
        if (activeThemeProvider == null) {
            String activeThemeName = settingsManager.getActiveTheme();
            System.out.println("active theme: " + activeThemeName);
            ArrayList<ThemeProvider> themes = new ArrayList<>(pluginManager.getExtensions(ThemeProviderHook.class));
            activeThemeProvider = themes.stream()
                    .filter(t -> t.getName().equals(activeThemeName))
                    .findFirst()
                    .orElse(getOSDefaultThemeProvider());
        }
        return activeThemeProvider;
    }

    private ThemeProvider getOSDefaultThemeProvider() {
        String osName = System.getProperty("os.name").toLowerCase();
        if (isWindows()) {
            return new LightWin11ThemeProvider();
        } else if (isMac()) {
            return new LightMacosThemeProvider();
        } else if (isLinux()) {
            String command = "echo $XDG_CURRENT_DESKTOP";  // Alternatively, you might use `echo $DESKTOP_SESSION`
            StringBuilder output = new StringBuilder();
            try {
                ProcessBuilder processBuilder = new ProcessBuilder("bash", "-c", command);
                Process process = processBuilder.start();
                BufferedReader reader = new BufferedReader(new InputStreamReader(process.getInputStream()));
                String line;
                while ((line = reader.readLine()) != null) {
                    output.append(line);
                }
                process.waitFor();
            } catch (IOException | InterruptedException e) {
                LOGGER.info("Unknown linux Desktop Environment found.");
                return new LightLegacyThemeProvider();
            }
            System.out.println(output.toString().trim());
        }

        return new LightLegacyThemeProvider();
    }

    public String getCssPath() {
        File cssFile = new File(DirectoryUtils.getTargetDirectoryPath() + File.separator + CSS_FILE_NAME);
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
        soyEvents.fire(new SettingsUpdatedEvent(false));
    }
}
