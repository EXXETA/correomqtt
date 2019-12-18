package com.exxeta.correomqtt.plugin.manager;

import lombok.AccessLevel;
import lombok.Getter;
import lombok.Setter;
import org.pf4j.Plugin;
import org.pf4j.PluginWrapper;

import java.security.Permissions;

public class PermissionPlugin extends Plugin {

    @Getter
    @Setter(AccessLevel.PACKAGE)
    private String pluginConfigFolder;

    public PermissionPlugin(PluginWrapper wrapper) {
        super(wrapper);
    }

    @SuppressWarnings("WeakerAccess") // needs to be protected for plugins being able to override it
    protected Permissions getPermissions() {
        return new Permissions();
    }
}
