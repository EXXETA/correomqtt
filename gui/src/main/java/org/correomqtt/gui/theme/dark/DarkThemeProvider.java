package org.correomqtt.gui.theme.dark;

import javafx.scene.paint.Color;
import org.correomqtt.gui.theme.BaseThemeProvider;
import org.correomqtt.gui.theme.IconMode;
import org.correomqtt.gui.theme.ThemeProvider;
import org.correomqtt.gui.plugin.spi.ThemeProviderHook;
import org.pf4j.Extension;

@Extension
public class DarkThemeProvider extends BaseThemeProvider implements ThemeProvider, ThemeProviderHook {

    @Override
    public String getName() {
        return "Dark";
    }

    @Override
    public String getCss() {
        return getCssFromInputStream(DarkThemeProvider.class.getResourceAsStream("dark.css"));
    }

    @Override
    public IconMode getIconMode() {
        return IconMode.WHITE;
    }

    @Override
    public Color getBackgroundColor() {
        return Color.web("#313131");
    }
}
