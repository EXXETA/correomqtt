package org.correomqtt.gui.theme.dark;

import org.correomqtt.plugin.spi.ThemeProviderHook;
import org.correomqtt.gui.theme.BaseThemeProvider;
import org.correomqtt.gui.theme.IconMode;
import org.correomqtt.gui.theme.ThemeProvider;
import org.pf4j.Extension;

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
