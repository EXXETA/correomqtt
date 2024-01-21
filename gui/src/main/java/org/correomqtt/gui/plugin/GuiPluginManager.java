package org.correomqtt.gui.plugin;

import org.correomqtt.core.fileprovider.PluginConfigProvider;
import org.correomqtt.core.plugin.PluginManager;
import org.correomqtt.gui.plugin.spi.DetailViewManipulatorHook;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import javax.inject.Inject;
import java.util.List;
import java.util.Objects;

public class GuiPluginManager {

    private static final Logger LOGGER = LoggerFactory.getLogger(GuiPluginManager.class);
    private final PluginManager pluginManager;

    @Inject
    public GuiPluginManager(PluginManager pluginManager){

        this.pluginManager = pluginManager;
    }

    public List<DetailViewManipulatorTask> getDetailViewManipulatorTasks() {
        return PluginConfigProvider.getInstance().getDetailViewTasks()
                .stream()
                .map(detailViewTaskDefinition -> {
                    List<DetailViewManipulatorHook> hooks = detailViewTaskDefinition.getExtensions().stream()
                            .map(extensionDefinition -> {
                                String pluginId = extensionDefinition.getPluginId();
                                String extensionId = extensionDefinition.getId();
                                DetailViewManipulatorHook extension = pluginManager.getExtensionById(DetailViewManipulatorHook.class, pluginId, extensionId);
                                if (extension == null) {
                                    LOGGER.warn("Plugin extension {}:{} in detailViewTasks is configured, but does not exist.", pluginId, extensionId);
                                    return null;
                                }
                                pluginManager.enrichExtensionWithConfig(extension, extensionDefinition.getConfig());
                                return extension;
                            })
                            .filter(Objects::nonNull)
                            .toList();

                    return DetailViewManipulatorTask.builder()
                            .name(detailViewTaskDefinition.getName())
                            .hooks(hooks)
                            .build();
                })
                .toList();
    }
}
