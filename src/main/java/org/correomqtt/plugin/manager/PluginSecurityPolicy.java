package org.correomqtt.plugin.manager;

import org.slf4j.LoggerFactory;

import java.io.FilePermission;
import java.lang.reflect.ReflectPermission;
import java.security.*;
import java.util.Arrays;
import java.util.HashMap;
import java.util.Iterator;

public class PluginSecurityPolicy extends Policy {

    // see https://tersesystems.com/blog/2015/12/29/sandbox-experiment/
    private static final Permission[] FORBIDDEN_PERMISSIONS = {
            new RuntimePermission("createClassLoader"),
            new RuntimePermission("accessClassInPackage.sun"),
            new RuntimePermission("setSecurityManager"),
            new ReflectPermission("suppressAccessChecks"),
            // there is a known use case where a plugin needs to write anywhere into the filesystem
            // new FilePermission("<<ALL FILES>>", "write, execute"),
            new SecurityPermission("setPolicy"),
            new SecurityPermission("setProperty.package.access")
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

    public static Permissions removeForbiddenPermissions(String pluginName, Permissions permissions) {
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

    private static boolean isPermissionAllowed(Permission permission) {
        if (permission instanceof FilePermission && permission.getActions().contains("execute")) return false;
        return Arrays.stream(FORBIDDEN_PERMISSIONS).noneMatch(p -> p.getClass().equals(permission.getClass())
                && p.getName().equals(permission.getName()));
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
