package org.correomqtt.gui.plugin;

import org.correomqtt.core.plugin.PluginManager;
import org.correomqtt.di.SoyDi;
import org.pf4j.ExtensionFactory;
import org.pf4j.PluginRuntimeException;
import org.pf4j.PluginWrapper;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class CorreoExtensionFactory implements ExtensionFactory {

    private static final Logger LOGGER = LoggerFactory.getLogger(CorreoExtensionFactory.class);

    @SuppressWarnings("unchecked")
    @Override
    public <T> T create(Class<T> extensionClass) {
        try {
            String pkg = extensionClass.getPackageName();
            if (!pkg.startsWith("org.correomqtt.core") && !pkg.startsWith("org.correomqtt.gui") && !SoyDi.isInjectable(extensionClass)) {
                PluginManager pluginManager = SoyDi.inject(PluginManager.class);
                PluginWrapper plugin = pluginManager.whichPlugin(extensionClass);
                if (plugin == null) {
                    throw new IllegalStateException("This class is not part of a plugin and could not be loaded as plugin: " + extensionClass);
                }
                SoyDi.addClassLoader(plugin.getPluginClassLoader());
                SoyDi.scan(pkg, false);
            }
            if (SoyDi.isInjectable(extensionClass)) {
                LOGGER.debug("Injecting Plugin Class {} with DI.", extensionClass.getName());
                return SoyDi.inject(extensionClass);
            } else {
                LOGGER.debug("Injecting Plugin Class {} without DI.", extensionClass.getName());
                return extensionClass.getDeclaredConstructor().newInstance();
            }
        } catch (Exception e) {
            throw new PluginRuntimeException(e);
        }
    }
}