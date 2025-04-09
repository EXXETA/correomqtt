package org.correomqtt.gui.theme.dark_legacy;

import javafx.scene.paint.Color;
import org.correomqtt.gui.theme.BaseThemeProvider;
import org.correomqtt.gui.theme.IconMode;
import org.correomqtt.gui.theme.ThemeProvider;
import org.correomqtt.gui.plugin.spi.ThemeProviderHook;
import org.pf4j.Extension;

@Extension
public class DarkLegacyThemeProvider extends BaseThemeProvider implements ThemeProvider, ThemeProviderHook {

    @Override
    public String getName() {
        return "Dark Legacy";
    }

    @Override
    public String getCss() {
        return getCssFromInputStream(DarkLegacyThemeProvider.class.getResourceAsStream("dark_legacy.css"));
    }

    @Override
    public IconMode getIconMode() {
        return IconMode.WHITE;
    }

    @Override
    public Color getBackgroundColor() {
        return Color.web("#282828");
    }
}
