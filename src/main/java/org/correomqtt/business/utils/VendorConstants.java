package org.correomqtt.business.utils;

public class VendorConstants {

    private VendorConstants(){
        // private constructor
    }

    //public static final String PLUGIN_REPO_URL =
    //public static final String DEFAULT_REPO_URL = "https://raw.githubusercontent.com/exxeta/ceomqtt-pluginrepo/master/plugins-" + VersionUtils.getVersion().trim() + ".json";
    //public static final String DEFAULT_REPO_URL = "http://localhost/correo-test.json";
    //public static final String BUNDLED_PLUGINS_URL = "http://localhost/correo-bundled.json";
    public static final String BUNDLED_PLUGINS_URL = "file:///opt/private/projects/concrete9test/www/public/correo-bundled.json";
    public static final String DEFAULT_REPO_URL = "file:///opt/private/projects/concrete9test/www/public/correo-local.json";
    public static final String WEBSITE = "http://correomqtt.org";
    public static final String GITHUB_API_LATEST = "https://api.github.com/repos/exxeta/correomqtt/releases/latest";
    public static final String GITHUB_LATEST = "https://github.com/EXXETA/correomqtt/releases/latest";
    public static final String UNINSTALL_VERSION = "0.0.0";
}
