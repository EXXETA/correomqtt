package org.correomqtt.plugin.manager;

import com.exxeta.correomqtt.business.services.ConfigService;
import org.pf4j.DefaultPluginFactory;
import org.pf4j.Plugin;
import org.pf4j.PluginWrapper;

import java.io.File;
import java.io.FilePermission;
import java.security.Permissions;
import java.security.Policy;

public class PermissionPluginFactory extends DefaultPluginFactory {

    private PluginSecurityPolicy pluginSecurityPolicy;

    PermissionPluginFactory() {
        this.pluginSecurityPolicy = (PluginSecurityPolicy) Policy.getPolicy();
    }

    @Override
    public Plugin create(PluginWrapper pluginWrapper) {
        Plugin plugin = super.create(pluginWrapper);

        if (plugin instanceof PermissionPlugin) {
            addPluginPermissions(pluginWrapper.getPluginId(), (PermissionPlugin) plugin);
        }

        return plugin;
    }

    private void addPluginPermissions(String pluginId, PermissionPlugin permissionPlugin) {
        String pluginConfigFolder = ConfigService.getInstance().getPluginConfigPath(pluginId);
        pluginSecurityPolicy.addPluginPermission(pluginId, getPluginConfigFolderPermission(pluginConfigFolder));
        permissionPlugin.setPluginConfigFolder(pluginConfigFolder);

        pluginSecurityPolicy.addPluginPermissions(pluginId, permissionPlugin.getPermissions());
    }

    private FilePermission getPluginConfigFolderPermission(String pluginConfigFolder) {
        return new FilePermission(pluginConfigFolder + File.separator + "-", "read, write");
    }
}
