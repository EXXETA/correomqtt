package org.correomqtt.gui.icons;

import org.kordamp.ikonli.AbstractIkonHandler;
import org.kordamp.ikonli.Ikon;

import java.io.InputStream;
import java.net.URL;

public class MaterialDesignIconHandler extends AbstractIkonHandler {
    private static final String FONT_RESOURCE = "/META-INF/resources/MaterialDesignIcons.ttf";

    @Override
    public boolean supports(String description) {
        return description != null && description.startsWith("mdi-");
    }

    @Override
    public Ikon resolve(String description) {
        return MaterialDesignIcon.findByDescription(description);
    }

    @Override
    public URL getFontResource() {
        return getClass().getResource(FONT_RESOURCE);
    }

    @Override
    public InputStream getFontResourceAsStream() {
        return getClass().getResourceAsStream(FONT_RESOURCE);
    }

    @Override
    public String getFontFamily() {
        return "Material Design Icons";
    }
}
