package org.correomqtt.gui.plugin;

import org.correomqtt.CorreoAppComponent;

public interface ExtensionInjector<T> {

    T create(CorreoAppComponent correoAppComponent);

}