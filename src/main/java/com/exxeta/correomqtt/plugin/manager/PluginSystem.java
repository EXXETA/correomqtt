package com.exxeta.correomqtt.plugin.manager;

import com.exxeta.correomqtt.business.services.ConfigService;
import com.exxeta.correomqtt.plugin.spi.BaseExtensionPoint;
import com.exxeta.correomqtt.plugin.spi.ExtensionId;
import org.jdom2.JDOMException;
import org.pf4j.*;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;
import java.nio.file.Path;
import java.util.*;
import java.util.stream.Collectors;

public class PluginSystem extends DefaultPluginManager {

    private static final Logger LOGGER = LoggerFactory.getLogger(PluginSystem.class);

    private static PluginSystem instance;

    private PluginProtocolParser pluginProtocolParser;

    private final HashMap<String, List> extensionsCache = new HashMap<>();
    private final HashMap<String, List<Task>> taskCache = new HashMap<>();

    private PluginSystem() {
        // private constructor
        super(Path.of(ConfigService.getInstance().getPluginJarPath()));
        try {
            pluginProtocolParser = new PluginProtocolParser();
        } catch (IOException | JDOMException e) {
            LOGGER.error("Cant't parse the protocol, please check the protocol.xml file.");
        }
    }

    @Override
    protected PluginFactory createPluginFactory() {
        return new PermissionPluginFactory();
    }

    @Override
    protected PluginLoader createPluginLoader() {
        // load only jar plugins
        return new PermissionJarPluginLoader(this);
    }

    @Override
    protected PluginDescriptorFinder createPluginDescriptorFinder() {
        // read plugin descriptor from jar's manifest
        return new ManifestPluginDescriptorFinder();
    }

    @Override
    protected ExtensionFactory createExtensionFactory() {
        return new PluginExtensionFactory();
    }

    public static PluginSystem getInstance() {
        if (instance == null) {
            instance = new PluginSystem();
        }
        return instance;
    }

    @Override
    public <T> List<T> getExtensions(Class<T> type) {
        if (!extensionsCache.containsKey(type.getSimpleName())) {
            if (pluginProtocolParser == null) {
                extensionsCache.put(type.getSimpleName(), super.getExtensions(type));
            } else {
                List<ProtocolExtensionPoint<T>> declaredExtensionsForClass = pluginProtocolParser.getProtocolExtensionPoints(type);
                if (declaredExtensionsForClass.isEmpty()) {
                    extensionsCache.put(type.getSimpleName(), super.getExtensions(type));
                } else {
                    extensionsCache.put(type.getSimpleName(), createExtensions(type, declaredExtensionsForClass));
                }
            }
        }

        return extensionsCache.get(type.getSimpleName());
    }

    public <T> List<Task<T>> getTasks(Class<T> type) {
        if (!taskCache.containsKey(type.getSimpleName())) {
            if (pluginProtocolParser == null) {
                taskCache.put(type.getSimpleName(), Collections.emptyList());
            } else {
                List<ProtocolTask<T>> declaredTasks = pluginProtocolParser.getDeclaredTasks(type);
                if (declaredTasks.isEmpty()) {
                    taskCache.put(type.getSimpleName(), Collections.emptyList());
                } else {
                    taskCache.put(type.getSimpleName(), declaredTasks
                            .stream()
                            .map(t -> createTask(type, t))
                            .filter(Objects::nonNull)
                            .collect(Collectors.toList()));
                }
            }
        }

        return taskCache.get(type.getSimpleName()).stream().map(t -> (Task<T>) t).collect(Collectors.toList());
    }

    private <T> Task<T> createTask(Class<T> type, ProtocolTask<T> protocolTask) {
        List<T> extensions = createExtensions(type, protocolTask.getTasks());
        if (extensions.size() == protocolTask.getTasks().size()) {
            return new Task<>(protocolTask.getId(), extensions);
        } else {
            LOGGER.warn("Can't find all declared extensions for task {} in {}", protocolTask.getId(), type.getSimpleName());
            return null;
        }
    }

    private <T> List<T> createExtensions(Class<T> type, List<ProtocolExtensionPoint<T>> declaredExtensionsForClass) {
        return declaredExtensionsForClass
                .stream()
                .map(pep -> createExtensionWithConfig(type, pep))
                .filter(Objects::nonNull)
                .collect(Collectors.toList());
    }

    private <T> T createExtensionWithConfig(Class<T> type, ProtocolExtensionPoint<T> pep) {
        T baseExtensionPoint = getExtensionById(type, pep.getPluginName(), pep.getExtensionId());
        if (baseExtensionPoint != null) {
            ((BaseExtensionPoint) baseExtensionPoint).onConfigReceived(pep.getPluginConfig());
        }
        return baseExtensionPoint;
    }

    private <T> T getExtensionById(Class<T> type, String pluginId, String extensionId) {
        return super.getExtensions(type, pluginId)
                .stream()
                .filter(e -> isExtensionIdResolved(e, extensionId))
                .findFirst()
                .orElseGet(() -> {
                    logInvalidPluginDeclaration(type, pluginId, extensionId);
                    return null;
                });
    }

    private <T> boolean isExtensionIdResolved(T e, String id) {
        if (e.getClass().isAnnotationPresent(ExtensionId.class)) {
            return e.getClass().getAnnotation(ExtensionId.class).value().equals(id);
        } else return true;
    }

    private <T> void logInvalidPluginDeclaration(Class<T> type, String pluginId, String extensionId) {
        Optional<T> defaultExtension = super.getExtensions(type, pluginId).stream().findFirst();
        if (defaultExtension.isPresent()) {
            if (extensionId == null) {
                LOGGER.info("Plugin {} declared for {} offers multiple valid extensions, please specify an extensionId", pluginId, type.getSimpleName());
            } else {
                LOGGER.info("Plugin {} declared for {} has no extension named: {}", pluginId, type.getSimpleName(), extensionId);
            }
        } else {
            if (getPlugin(pluginId).getPluginState().equals(PluginState.STARTED)) {
                LOGGER.warn("Plugin {} declared for {} has no valid extension", pluginId, type.getSimpleName());
            } else {
                LOGGER.warn("Plugin {} declared for {} is not started", pluginId, type.getSimpleName());
            }
        }
    }
}
