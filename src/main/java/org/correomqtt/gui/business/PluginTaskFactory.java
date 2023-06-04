package org.correomqtt.gui.business;

import lombok.extern.slf4j.Slf4j;
import org.correomqtt.business.services.PluginDisableService;
import org.correomqtt.business.services.PluginEnableService;
import org.correomqtt.business.services.PluginInstallService;
import org.correomqtt.business.services.PluginUninstallService;

@Slf4j
public class PluginTaskFactory {

    private PluginTaskFactory() {
        // private constructor
    }

    public static void install(String pluginId, String version) {
        new GuiService<>(new PluginInstallService(pluginId, version),
                PluginInstallService::install).start();
    }

    public static void uninstall(String pluginId) {
        new GuiService<>(new PluginUninstallService(pluginId),
                PluginUninstallService::uninstall).start();
    }

    public static void enable(String pluginId) {
        new GuiService<>(new PluginEnableService(pluginId),
                PluginEnableService::enable).start();
    }

    public static void disable(String pluginId) {
        new GuiService<>(new PluginDisableService(pluginId),
                PluginDisableService::disable).start();
    }
}
