package org.correomqtt.gui.plugin;

import org.correomqtt.di.SoyDi;
import org.pf4j.ExtensionFactory;
import org.pf4j.PluginRuntimeException;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class CorreoExtensionFactory implements ExtensionFactory {

    private static final Logger LOGGER = LoggerFactory.getLogger(CorreoExtensionFactory.class);

    @SuppressWarnings("unchecked")
    @Override
    public <T> T create(Class<T> extensionClass) {
        try {
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