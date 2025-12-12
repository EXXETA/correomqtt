package org.correomqtt.gui.theme.dark_breeze;

import javafx.scene.paint.Color;
import org.correomqtt.gui.theme.BaseThemeProvider;
import org.correomqtt.gui.theme.IconMode;
import org.correomqtt.gui.theme.ThemeProvider;
import org.correomqtt.gui.plugin.spi.ThemeProviderHook;
import org.pf4j.Extension;

@Extension
public class DarkBreezeThemeProvider extends BaseThemeProvider implements ThemeProvider, ThemeProviderHook {

    @Override
    public String getName() {
        return "Dark Breeze";
    }

    @Override
    public String getCss() {
        System.out.println("DTP");
        return getCssFromInputStream(DarkBreezeThemeProvider.class.getResourceAsStream("dark_breeze.css"));
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
