package org.correomqtt.gui.theme.dark_win11;

import javafx.scene.paint.Color;
import org.correomqtt.gui.plugin.spi.ThemeProviderHook;
import org.correomqtt.gui.theme.BaseThemeProvider;
import org.correomqtt.gui.theme.IconMode;
import org.correomqtt.gui.theme.ThemeProvider;
import org.pf4j.Extension;

@Extension
public class DarkWin11ThemeProvider extends BaseThemeProvider implements ThemeProvider, ThemeProviderHook {
    @Override
    public String getName() {
        return "Dark Windows";
    }

    @Override
    public String getCss() {
        return getCssFromInputStream(DarkWin11ThemeProvider.class.getResourceAsStream("dark_win11.css"));
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
