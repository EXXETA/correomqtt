package org.correomqtt.business.utils;

import lombok.Getter;
import lombok.experimental.Accessors;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.HashMap;
import java.util.Map;
import java.util.Objects;

public class VendorConstants {

    private static final Logger LOGGER = LoggerFactory.getLogger(VendorConstants.class);

    private VendorConstants() {
        // private constructor
    }

    private static final Map<String, String> CACHE = new HashMap<>();

    private static String getFromCache(String envOverride, String defaultValue) {
        if (!CACHE.containsKey(envOverride)) {
            String env = System.getenv(envOverride);
            LOGGER.error("Override {} via ENV: {}", envOverride, env);
            CACHE.put(envOverride, Objects.requireNonNullElse(env, defaultValue));
        }
        return CACHE.get(envOverride);
    }

    public static String BUNDLED_PLUGINS_URL() {
        return getFromCache(BUNDLED_PLUGINS_URL_ENV_OVERRIDE, BUNDLED_PLUGINS_URL_DEFAULT);
    }

    public static String DEFAULT_REPO_URL() {
        return getFromCache(DEFAULT_REPO_URL_ENV_OVERRIDE, DEFAULT_REPO_URL_DEFAULT);
    }

    private static final String BUNDLED_PLUGINS_URL_ENV_OVERRIDE = "CORREO_BUNDLED_PLUGINS_URL";
    private static final String BUNDLED_PLUGINS_URL_DEFAULT = "https://github.com/EXXETA/correomqtt-pluginrepo/blob/master/bundled.json";
    private static final String DEFAULT_REPO_URL_ENV_OVERRIDE = "CORREO_DEFAULT_REPO_URL";
    private static final String DEFAULT_REPO_URL_DEFAULT = "https://github.com/EXXETA/correomqtt-pluginrepo/blob/master/default-repo.json";

    @Getter
    @Accessors(fluent = true)
    private static final String WEBSITE = "http://correomqtt.org";

    @Getter
    @Accessors(fluent = true)
    private static final String GITHUB_API_LATEST = "https://api.github.com/repos/exxeta/correomqtt/releases/latest";

    @Getter
    @Accessors(fluent = true)
    private static final String GITHUB_LATEST = "https://github.com/EXXETA/correomqtt/releases/latest";

}
