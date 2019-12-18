package com.exxeta.correomqtt.plugin.manager;

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

        addPluginPermissions(pluginWrapper.getPluginId(), plugin);

        return plugin;
    }

    private void addPluginPermissions(String pluginId, Plugin plugin) {
        String pluginConfigFolder = ConfigService.getInstance().getPluginConfigPath(pluginId);
        pluginSecurityPolicy.addPluginPermission(pluginId, getPluginConfigFolderPermission(pluginConfigFolder));

        if (plugin instanceof PermissionPlugin) {
            PermissionPlugin permissionPlugin = (PermissionPlugin) plugin;
            permissionPlugin.setPluginConfigFolder(pluginConfigFolder);
            Permissions pluginPermissions = permissionPlugin.getPermissions();
            pluginSecurityPolicy.addPluginPermissions(pluginId, pluginPermissions);
        }
    }

    private FilePermission getPluginConfigFolderPermission(String pluginConfigFolder) {
        return new FilePermission(pluginConfigFolder + File.separator + "-", "read, write");
    }
}
