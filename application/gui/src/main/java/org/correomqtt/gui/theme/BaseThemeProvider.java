package org.correomqtt.gui.theme;


import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.io.Reader;
import java.nio.charset.StandardCharsets;

public abstract class BaseThemeProvider implements ThemeProvider {

    private static final Logger LOGGER = LoggerFactory.getLogger(BaseThemeProvider.class);

    protected String getCssFromInputStream(InputStream inputStream) {
        StringBuilder css = new StringBuilder();
        try {
            try (Reader reader = new BufferedReader(new InputStreamReader(inputStream, StandardCharsets.UTF_8))) {
                int c;
                while ((c = reader.read()) != -1) {
                    css.append((char) c);
                }
            }
        } catch (IOException e) {
            LOGGER.info("Error reading theme", e);
        }

        return css.toString();
    }
}

