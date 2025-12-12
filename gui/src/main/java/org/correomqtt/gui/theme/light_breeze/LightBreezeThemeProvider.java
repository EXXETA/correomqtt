package org.correomqtt.gui.theme.light_breeze;

import javafx.scene.paint.Color;
import org.correomqtt.gui.plugin.spi.ThemeProviderHook;
import org.correomqtt.gui.theme.BaseThemeProvider;
import org.correomqtt.gui.theme.IconMode;
import org.correomqtt.gui.theme.ThemeProvider;
import org.pf4j.Extension;

@Extension
public class LightBreezeThemeProvider extends BaseThemeProvider implements ThemeProvider, ThemeProviderHook {

    @Override
    public String getName() {
        return "Light Breeze";
    }

    @Override
    public String getCss() {
        return getCssFromInputStream(LightBreezeThemeProvider.class.getResourceAsStream("light_breeze.css"));
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
