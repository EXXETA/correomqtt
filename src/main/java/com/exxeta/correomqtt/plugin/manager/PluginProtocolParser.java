package com.exxeta.correomqtt.plugin.manager;

import com.exxeta.correomqtt.business.services.ConfigService;
import com.exxeta.correomqtt.plugin.spi.BaseExtensionPoint;
import com.exxeta.correomqtt.plugin.spi.ExtensionId;
import org.jdom2.Document;
import org.jdom2.Element;
import org.jdom2.JDOMException;
import org.jdom2.input.SAXBuilder;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.*;
import java.util.Collections;
import java.util.List;
import java.util.Objects;
import java.util.Optional;
import java.util.stream.Collectors;

class PluginProtocolParser {

    private static final Logger LOGGER = LoggerFactory.getLogger(PluginProtocolParser.class);

    private static final String XML_TAG_LISTS = "lists";
    private static final String XML_TAG_TASKS = "tasks";
    private static final String XML_ATTR_NAME = "name";
    private static final String XML_ATTR_EXTENSION_ID = "extensionId";
    private static final String XML_ATTR_ID = "id";

    private final PluginSystem pluginSystem;
    private final Element protocol;

    PluginProtocolParser(PluginSystem pluginSystem) throws IOException, JDOMException {
        this.pluginSystem = pluginSystem;
        this.protocol = parsePluginProtocol();
    }

    private Element parsePluginProtocol() throws IOException, JDOMException {
        File protocolFile = new File(ConfigService.getInstance().getPluginProtocol());
        if (!protocolFile.exists()) {
            createDefaultProtocolFile(protocolFile);
        }
        SAXBuilder builder = new SAXBuilder();
        Document document = builder.build(protocolFile);
        return document.getRootElement();
    }

    @SuppressWarnings("ResultOfMethodCallIgnored")
    private void createDefaultProtocolFile(File protocolFile) throws IOException {
        try (InputStream inputStream = PluginProtocolParser.class.getResourceAsStream(protocolFile.getName())) {
            byte[] buffer = new byte[inputStream.available()];
            inputStream.read(buffer);

            try (OutputStream outStream = new FileOutputStream(protocolFile)) {
                outStream.write(buffer);
            }
        }
    }

    <T> List<PluginProtocolTask<T>> getDeclaredTasks(Class<T> type) {
        if (protocol == null) return List.of();

        List<PluginProtocolTask<?>> tasks = protocol.getChild(XML_TAG_TASKS).getChild(type.getSimpleName()).getChildren().stream()
                .map(declaredTask -> createTypedTask(type, declaredTask))
                .filter(Objects::nonNull)
                .collect(Collectors.toList());
        return tasks.stream().map(t -> (PluginProtocolTask<T>) t).collect(Collectors.toList());
    }

    private <T> PluginProtocolTask<T> createTypedTask(Class<T> type, Element declaredTask) {
        String taskId = declaredTask.getAttributeValue(XML_ATTR_ID);
        List<T> tasks = getDeclaredExtensions(type, declaredTask);
        if (!tasks.isEmpty()) {
            if (tasks.size() == declaredTask.getChildren().size()) {
                return new PluginProtocolTask<>(taskId, tasks);
            } else {
                LOGGER.warn("Can't find all declared tasks for {} in {}", taskId, type.getSimpleName());
            }
        }
        return null;
    }

    <T> List<T> getDeclaredExtensions(Class<T> type) {
        if (protocol == null) return Collections.emptyList();

        Element pluginsForClass = protocol.getChild(XML_TAG_LISTS).getChild(type.getSimpleName());
        return getDeclaredExtensions(type, pluginsForClass);
    }

    private <T> List<T> getDeclaredExtensions(Class<T> type, Element pluginsForType) {
        List<ProtocolExtensionPoint> declaredExtensionsForClass = getProtocolExtensionPoints(pluginsForType);
        if (declaredExtensionsForClass.isEmpty()) {
            return Collections.emptyList();
        }

        return declaredExtensionsForClass
                .stream()
                .map(pep -> createTypedExtension(type, pep))
                .filter(Objects::nonNull)
                .collect(Collectors.toList());
    }

    private List<ProtocolExtensionPoint> getProtocolExtensionPoints(Element pluginsForType) {
        if (pluginsForType == null) return List.of();

        return pluginsForType.getChildren()
                .stream()
                .map(this::getProtocolExtensionPoint)
                .collect(Collectors.toList());
    }

    private ProtocolExtensionPoint getProtocolExtensionPoint(Element pluginElement) {
        String name = pluginElement.getAttributeValue(XML_ATTR_NAME);
        String extensionId = pluginElement.getAttributeValue(XML_ATTR_EXTENSION_ID);
        return new ProtocolExtensionPoint(name, extensionId, pluginElement);
    }

    private <T> T createTypedExtension(Class<T> type, ProtocolExtensionPoint pep) {
        T baseExtensionPoint = getExtensionById(type, pep.getPluginName(), pep.getExtensionId()).orElse(null);
        if (baseExtensionPoint != null) {
            ((BaseExtensionPoint) baseExtensionPoint).onConfigReceived(pep.getPluginConfig());
        }
        return baseExtensionPoint;
    }

    private <T> Optional<T> getExtensionById(Class<T> type, String pluginId, String extensionId) {
        return pluginSystem.getExtensions(type, pluginId)
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
        Optional<T> defaultExtension = pluginSystem.getExtensions(type, pluginId).stream().findFirst();
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
