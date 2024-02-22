package org.correomqtt.core.utils;

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
        return CACHE.computeIfAbsent(envOverride, k -> {
            String env = System.getenv(envOverride);
            if (env != null) {
                LOGGER.warn("Override {} via ENV: {}", envOverride, env);
            }
            return Objects.requireNonNullElse(env, defaultValue);
        });
    }

    public static String getBundledPluginsUrl() {
        return getFromCache(BUNDLED_PLUGINS_URL_ENV_OVERRIDE, BUNDLED_PLUGINS_URL_DEFAULT);
    }

    public static String getDefaultRepoUrl() {
        return getFromCache(DEFAULT_REPO_URL_ENV_OVERRIDE, DEFAULT_REPO_URL_DEFAULT);
    }

    public static String getGithubApiLatest() {
        return getFromCache(GITHUB_API_LATEST_ENV_OVERRIDE, GITHUB_API_LATEST);
    }

    public static String getGithubLatest() {
        return getFromCache(GITHUB_LATEST_ENV_OVERRIDE, GITHUB_LATEST);
    }

    @Getter
    @Accessors(fluent = true)
    private static final String BUNDLED_PLUGINS_URL_ENV_OVERRIDE = "CORREO_BUNDLED_PLUGINS_URL";
    private static final String BUNDLED_PLUGINS_URL_DEFAULT = "https://github.com/EXXETA/correomqtt/releases/download/latest/bundled.json";
    @Getter
    @Accessors(fluent = true)
    private static final String DEFAULT_REPO_URL_ENV_OVERRIDE = "CORREO_DEFAULT_REPO_URL";
    private static final String DEFAULT_REPO_URL_DEFAULT = "https://github.com/EXXETA/correomqtt/releases/download/latest/default-repo.json";

    @Getter
    @Accessors(fluent = true)
    private static final String WEBSITE = "http://correomqtt.org";

    @Getter
    @Accessors(fluent = true)
    private static final String GITHUB_API_LATEST_ENV_OVERRIDE = "CORREO_GITHUB_API_LATEST_URL";

    private static final String GITHUB_API_LATEST = "https://api.github.com/repos/exxeta/correomqtt/releases/latest";

    @Getter
    @Accessors(fluent = true)
    private static final String GITHUB_LATEST_ENV_OVERRIDE = "CORREO_GITHUB_LATEST_URL";

    private static final String GITHUB_LATEST = "https://github.com/EXXETA/correomqtt/releases/latest";
}
