package com.exxeta.correomqtt.plugin.manager;

import com.exxeta.correomqtt.business.services.ConfigService;
import com.exxeta.correomqtt.plugin.spi.BaseExtensionPoint;
import com.exxeta.correomqtt.plugin.spi.ExtensionId;
import org.jdom2.Element;
import org.jdom2.JDOMException;
import org.pf4j.*;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;
import java.nio.file.Path;
import java.util.Collections;
import java.util.List;
import java.util.Objects;
import java.util.Optional;
import java.util.stream.Collectors;

public class PluginSystem extends DefaultPluginManager {

    private static final Logger LOGGER = LoggerFactory.getLogger(PluginSystem.class);

    private static PluginSystem instance;

    private PluginProtocolParser pluginProtocolParser;

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
        if (pluginProtocolParser == null) return super.getExtensions(type);

        List<ProtocolExtensionPoint<T>> declaredExtensionsForClass = pluginProtocolParser.getProtocolExtensionPoints(type);
        if (declaredExtensionsForClass.isEmpty()) return super.getExtensions(type);

        return createExtensions(type, declaredExtensionsForClass);
    }

    public <T> List<Task<T>> getTasks(Class<T> type) {
        if (pluginProtocolParser == null) return Collections.emptyList();

        List<ProtocolTask<T>> declaredTasks = pluginProtocolParser.getDeclaredTasks(type);
        if (declaredTasks.isEmpty()) return Collections.emptyList();

        return declaredTasks
                .stream()
                .map(t -> {
                    List<T> extensions = createExtensions(type, t.getTasks());
                    if (extensions.size() == t.getTasks().size()) {
                        return new Task<>(t.getId(), extensions);
                    } else {
                        LOGGER.warn("Can't find all declared extensions for task {} in {}", t.getId(), type.getSimpleName());
                        return null;
                    }
                })
                .filter(Objects::nonNull)
                .collect(Collectors.toList());
    }

    private <T> List<T> createExtensions(Class<T> type, List<ProtocolExtensionPoint<T>> declaredExtensionsForClass) {
        return declaredExtensionsForClass
                .stream()
                .map(pep -> createTypedExtension(type, pep))
                .filter(Objects::nonNull)
                .collect(Collectors.toList());
    }

    private <T> T createTypedExtension(Class<T> type, ProtocolExtensionPoint<T> pep) {
        T baseExtensionPoint = getExtensionById(type, pep.getPluginName(), pep.getExtensionId()).orElse(null);
        if (baseExtensionPoint != null) {
            ((BaseExtensionPoint) baseExtensionPoint).onConfigReceived(pep.getPluginConfig());
        }
        return baseExtensionPoint;
    }

    private <T> Optional<T> getExtensionById(Class<T> type, String pluginId, String extensionId) {
        return super.getExtensions(type, pluginId)
                .stream()
                .filter(e -> hasExtensionId(e, extensionId))
                .findFirst()
                .or(() -> {
                    logInvalidPluginDeclaration(type, pluginId, extensionId);
                    return Optional.empty();
                });
    }

    private <T> boolean hasExtensionId(T e, String id) {
        if (e.getClass().isAnnotationPresent(ExtensionId.class)) {
            return e.getClass().getAnnotation(ExtensionId.class).value().equals(id);
        } else return true;
    }

    private <T> void logInvalidPluginDeclaration(Class<T> type, String pluginId, String extensionId) {
        Optional<T> defaultExtension = super.getExtensions(type, pluginId).stream().findFirst();
        if (defaultExtension.isPresent()) {
            if (extensionId == null) {
                LOGGER.info("Please specify an extensionId for {} declared for {}", pluginId, type.getSimpleName());
            } else {
                LOGGER.info("Extension {} not found in plugin {} declared for {}", extensionId, pluginId, type.getSimpleName());
            }
        } else {
            LOGGER.warn("Plugin {} declared for {} has no valid extension", pluginId, type.getSimpleName());
        }
    }
}
