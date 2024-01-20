package org.correomqtt.core.settings;

import org.correomqtt.core.model.ConnectionConfigDTO;

import java.util.List;
import java.util.Map;

public interface CoreSettings {
    boolean isInstallBundledPlugins();

    String getBundledPluginsUrl();

    boolean isSearchUpdates()
            ;

    boolean isUseDefaultRepo();

    Map<String, String> getPluginRepositories();

    List<ConnectionConfigDTO> getConnectionConfigs();
}
