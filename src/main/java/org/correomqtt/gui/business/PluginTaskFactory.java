package org.correomqtt.gui.business;

import lombok.extern.slf4j.Slf4j;
import org.correomqtt.business.services.ConnectService;
import org.correomqtt.business.services.DisconnectService;
import org.correomqtt.business.services.ExportMessageService;
import org.correomqtt.business.services.ImportMessageService;
import org.correomqtt.business.services.PluginDisableService;
import org.correomqtt.business.services.PluginEnableService;
import org.correomqtt.business.services.PluginInstallService;
import org.correomqtt.business.services.PluginUninstallService;
import org.correomqtt.business.services.PublishService;
import org.correomqtt.business.services.SubscribeService;
import org.correomqtt.business.services.UnsubscribeService;
import org.correomqtt.gui.model.MessagePropertiesDTO;
import org.correomqtt.gui.model.SubscriptionPropertiesDTO;
import org.correomqtt.gui.transformer.MessageTransformer;
import org.correomqtt.gui.transformer.SubscriptionTransformer;
import org.correomqtt.plugin.manager.PluginManager;
import org.correomqtt.plugin.model.MessageExtensionDTO;
import org.correomqtt.plugin.spi.OutgoingMessageHook;

import java.io.File;

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
