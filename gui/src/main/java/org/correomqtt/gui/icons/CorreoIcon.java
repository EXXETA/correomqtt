package org.correomqtt.gui.icons;

import lombok.AllArgsConstructor;
import lombok.Getter;
import org.kordamp.ikonli.Ikon;

import java.util.HashMap;
import java.util.Map;

@AllArgsConstructor
@Getter
public class CorreoIcon extends BaseIcon implements Ikon {

    private static final String JSON_RESOURCE = "/META-INF/resources/CorreoIcons.json";

    private static final Map<String, Integer> CACHE = new HashMap<>();

    public static CorreoIcon findByDescription(String description) {
        ensureCache(CACHE, JSON_RESOURCE);
        checkCache(CACHE, description);
        return new CorreoIcon(description, CACHE.get(description));
    }

    private final String description;
    private final int code;

}