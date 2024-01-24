package org.correomqtt.gui.plugin;

import org.correomqtt.CorreoAppComponent;
import org.correomqtt.FxApplication;
import org.pf4j.ExtensionFactory;
import org.pf4j.PluginRuntimeException;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class CorreoExtensionFactory implements ExtensionFactory {

    private static final Logger LOGGER = LoggerFactory.getLogger(CorreoExtensionFactory.class);

    @SuppressWarnings("unchecked")
    @Override
    public <T> T create(Class<T> extensionClass) {

        String componentClassName = extensionClass.getPackageName() + ".Dagger" + extensionClass.getSimpleName() + "_Factory";

        Class<?> factory = null;

        try {
            factory = Class.forName(componentClassName, false, extensionClass.getClassLoader());
            LOGGER.debug("{} exists, injecting {} with DI.", componentClassName, extensionClass.getName());
        } catch (Exception ignored) {
            LOGGER.debug("{} does not exist, injecting {} without DI.", componentClassName, extensionClass.getName());
        }

        try {
            if (factory != null) {
                Object builder1 = factory.getMethod("builder").invoke(null);
                Object builder2 = builder1.getClass().getMethod("correoAppComponent", CorreoAppComponent.class).invoke(builder1, FxApplication.getAppComponent());
                Object component = builder2.getClass().getMethod("build").invoke(builder2);
                return (T) ((ExtensionComponent) component).extension();
                // return (T) factory.getMethod("get").invoke(null);
            } else {
                return extensionClass.getDeclaredConstructor().newInstance();
            }
        } catch (Exception e) {
            throw new PluginRuntimeException(e);
        }
    }
}