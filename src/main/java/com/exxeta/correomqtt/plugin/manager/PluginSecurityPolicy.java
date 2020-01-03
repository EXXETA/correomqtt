package com.exxeta.correomqtt.plugin.manager;

import org.slf4j.LoggerFactory;

import java.io.FilePermission;
import java.security.*;
import java.util.Arrays;
import java.util.HashMap;
import java.util.Iterator;

public class PluginSecurityPolicy extends Policy {

    private static final String[] FORBIDDEN_PERMISSIONS = {
            "createClassLoader",
            "accessClassInPackage.sun",
            "setSecurityManager",
            "suppressAccessChecks",
            "setPolicy",
            "setProperty.package.access"
    };

    private HashMap<String, Permissions> pluginPermissions = new HashMap<>();

    void addPluginPermissions(String pluginName, Permissions permissions) {
        if (pluginPermissions.containsKey(pluginName)) {
            Permissions existingPermissions = pluginPermissions.get(pluginName);
            Iterator<Permission> permissionIterator = removeForbiddenPermissions(pluginName, permissions).elements().asIterator();
            while (permissionIterator.hasNext()) {
                Permission p = permissionIterator.next();
                existingPermissions.add(p);
            }
        } else {
            pluginPermissions.put(pluginName, removeForbiddenPermissions(pluginName, permissions));
        }
    }

    private Permissions removeForbiddenPermissions(String pluginName, Permissions permissions) {
        Permissions cleanPermissions = new Permissions();
        Iterator<Permission> permissionIterator = permissions.elements().asIterator();
        while (permissionIterator.hasNext()) {
            Permission p = permissionIterator.next();
            if (isPermissionAllowed(p)) {
                cleanPermissions.add(p);
            } else {
                LoggerFactory.getLogger(PluginSecurityPolicy.class).warn("{} tried to add forbidden permission: {}", pluginName, p);
            }
        }
        return cleanPermissions;
    }

    private boolean isPermissionAllowed(Permission permission) {
        if (permission instanceof FilePermission && permission.getActions().contains("execute")) return false;
        return !Arrays.asList(FORBIDDEN_PERMISSIONS).contains(permission.getName());
    }

    void addPluginPermission(String pluginName, Permission permission) {
        Permissions permissions = new Permissions();
        permissions.add(permission);
        addPluginPermissions(pluginName, permissions);
    }

    @Override
    public PermissionCollection getPermissions(ProtectionDomain domain) {
        if (isPlugin(domain)) {
            return pluginPermissions(((PermissionPluginClassLoader) domain.getClassLoader()).getPluginId());
        } else {
            return applicationPermissions();
        }
    }

    private boolean isPlugin(ProtectionDomain domain) {
        return domain.getClassLoader() instanceof PermissionPluginClassLoader;
    }

    private PermissionCollection pluginPermissions(String name) {
        return pluginPermissions.getOrDefault(name, new Permissions());
    }

    private PermissionCollection applicationPermissions() {
        Permissions permissions = new Permissions();
        permissions.add(new AllPermission());
        return permissions;
    }
}
