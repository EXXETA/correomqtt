package org.correomqtt.gui.theme.light_legacy;

import javafx.scene.paint.Color;
import org.correomqtt.gui.theme.BaseThemeProvider;
import org.correomqtt.gui.theme.IconMode;
import org.correomqtt.gui.theme.ThemeProvider;
import org.correomqtt.plugin.spi.ThemeProviderHook;
import org.pf4j.Extension;

@Extension
public class LightLegacyThemeProvider extends BaseThemeProvider implements ThemeProvider, ThemeProviderHook {

    @Override
    public String getName() {
        return "Light Legacy";
    }

    @Override
    public String getCss() {
        return getCssFromInputStream(LightLegacyThemeProvider.class.getResourceAsStream("light_legacy.css"));
    }

    @Override
    public IconMode getIconMode() {
        return IconMode.BLACK;
    }

    @Override
    public Color getBackgroundColor() {
        return Color.web("#ececec");
    }
}
