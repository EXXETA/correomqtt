package org.correomqtt.gui.icons;

import org.kordamp.ikonli.AbstractIkonHandler;
import org.kordamp.ikonli.Ikon;
import java.io.InputStream;
import java.net.URL;

public class CorreoIconHandler extends AbstractIkonHandler {
    private static final String FONT_RESOURCE = "/META-INF/resources/CorreoIcons.ttf";

    @Override
    public boolean supports(String description) {
        return description != null && description.startsWith("correo-");
    }

    @Override
    public Ikon resolve(String description) {
        return CorreoIcon.findByDescription(description);
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
        return "CorreoIcons";
    }
}
