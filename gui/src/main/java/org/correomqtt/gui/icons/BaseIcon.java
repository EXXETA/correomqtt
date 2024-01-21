package org.correomqtt.gui.icons;

import com.fasterxml.jackson.databind.ObjectMapper;

import java.io.IOException;
import java.util.Map;
import java.util.stream.Collectors;

abstract class BaseIcon {

    BaseIcon() {
        // package-private constructor
    }

    static void checkCache(Map<String, Integer> cache, String description) {
        if (!cache.containsKey(description)) {
            throw new IllegalArgumentException("Icon description '" + description + "' is invalid!");
        }
    }

    static void ensureCache(Map<String, Integer> cache, String jsonReference) {
        if (cache.isEmpty()) {
            try {
                loadCache(cache, jsonReference);
            } catch (IOException e) {
                throw new IllegalArgumentException("Unable to parse json dictionary.", e);
            }
        }
    }

    static void loadCache(Map<String, Integer> cache, String jsonReference) throws IOException {
        cache.putAll(new ObjectMapper()
                .readValue(CorreoIcon.class.getResource(jsonReference), Symbols.class)
                .getIcons()
                .stream()
                .collect(Collectors.toMap(
                        IconMap::getName,
                        im -> Integer.parseInt(im.getCode(), 16))));
    }
}
