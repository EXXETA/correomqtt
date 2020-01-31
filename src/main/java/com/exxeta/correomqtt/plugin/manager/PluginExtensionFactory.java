package com.exxeta.correomqtt.plugin.manager;

import org.pf4j.DefaultExtensionFactory;
import org.pf4j.Plugin;
import org.pf4j.PluginRuntimeException;

import java.lang.reflect.Constructor;

public class PluginExtensionFactory extends DefaultExtensionFactory {

    @Override
    public <T> T create(Class<T> extensionClass) {
        try {
            try {
                Constructor<T> constructor = extensionClass.getConstructor(Plugin.class);
                Plugin plugin = PluginSystem.getInstance().whichPlugin(extensionClass).getPlugin();
                return constructor.newInstance(plugin);
            } catch (Exception e) {
                return super.create(extensionClass);
            }
        } catch (Exception e) {
            throw new PluginRuntimeException(e);
        }
    }
}
