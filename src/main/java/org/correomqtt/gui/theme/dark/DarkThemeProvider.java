package org.correomqtt.gui.theme.dark;

import org.correomqtt.gui.theme.BaseThemeProvider;
import org.correomqtt.gui.theme.IconMode;
import org.correomqtt.gui.theme.ThemeProvider;
import org.correomqtt.gui.theme.light.LightThemeProvider;
import org.correomqtt.plugin.spi.ThemeProviderHook;
import org.pf4j.Extension;

import java.io.File;
import java.net.URL;

@Extension
public class DarkThemeProvider extends BaseThemeProvider implements ThemeProvider, ThemeProviderHook {

    @Override
    public String getName() {
        return "Dark";
    }

    @Override
    public String getCss() {
        return getCssFromInputStream(DarkThemeProvider.class.getResourceAsStream(getName() + ".css"));
    }

    @Override
    public IconMode getIconMode() {
        return IconMode.WHITE;
    }
}
