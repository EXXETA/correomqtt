package org.correomqtt.plugin.manager;

import lombok.Getter;
import org.pf4j.PluginClassLoader;
import org.pf4j.PluginDescriptor;
import org.pf4j.PluginManager;

class PermissionPluginClassLoader extends PluginClassLoader {

    @Getter
    private String pluginId;

    PermissionPluginClassLoader(PluginManager pluginManager, PluginDescriptor pluginDescriptor, ClassLoader parent) {
        super(pluginManager, pluginDescriptor, parent);
        this.pluginId = pluginDescriptor.getPluginId();
    }
}
