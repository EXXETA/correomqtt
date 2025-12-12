package org.correomqtt.gui.theme.dark_gnome;

import javafx.scene.paint.Color;
import org.correomqtt.gui.plugin.spi.ThemeProviderHook;
import org.correomqtt.gui.theme.BaseThemeProvider;
import org.correomqtt.gui.theme.IconMode;
import org.correomqtt.gui.theme.ThemeProvider;
import org.pf4j.Extension;

@Extension
public class DarkGnomeThemeProvider extends BaseThemeProvider implements ThemeProvider, ThemeProviderHook {

    @Override
    public String getName() {
        return "Dark Gnome";
    }

    @Override
    public String getCss() {
        System.out.println("DGTP");
        return getCssFromInputStream(DarkGnomeThemeProvider.class.getResourceAsStream("dark_gnome.css"));
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
