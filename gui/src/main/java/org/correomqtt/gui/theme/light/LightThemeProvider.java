package org.correomqtt.gui.theme.light;

import javafx.scene.paint.Color;
import org.correomqtt.gui.theme.BaseThemeProvider;
import org.correomqtt.gui.theme.IconMode;
import org.correomqtt.gui.theme.ThemeProvider;
import org.correomqtt.gui.plugin.spi.ThemeProviderHook;
import org.pf4j.Extension;

@Extension
public class LightThemeProvider extends BaseThemeProvider implements ThemeProvider, ThemeProviderHook {

    @Override
    public String getName() {
        return "Light";
    }

    @Override
    public String getCss() {
        return getCssFromInputStream(LightThemeProvider.class.getResourceAsStream("light.css"));
    }

    @Override
    public IconMode getIconMode() {
        return IconMode.BLACK;
    }

    @Override
    public Color getBackgroundColor() {
        return Color.web("#eff0f1");
    }
}
