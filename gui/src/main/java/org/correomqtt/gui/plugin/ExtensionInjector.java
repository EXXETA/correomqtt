package org.correomqtt.gui.plugin;

import org.correomqtt.MainComponent;

public interface ExtensionInjector<T> {

    T create(MainComponent correoAppComponent);

}