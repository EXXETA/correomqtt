package org.correomqtt.gui.theme;


import java.io.*;
import java.nio.charset.StandardCharsets;

public abstract class BaseThemeProvider implements ThemeProvider{


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
            e.printStackTrace(); //TODO
        }

        return css.toString();
    }
}

