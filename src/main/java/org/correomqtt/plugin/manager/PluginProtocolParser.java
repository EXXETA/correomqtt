package org.correomqtt.plugin.manager;

import org.correomqtt.business.provider.ConfigProvider;
import org.jdom2.Document;
import org.jdom2.Element;
import org.jdom2.JDOMException;
import org.jdom2.input.SAXBuilder;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.*;
import java.util.Collections;
import java.util.List;
import java.util.stream.Collectors;

class PluginProtocolParser {

    private static final Logger LOGGER = LoggerFactory.getLogger(PluginProtocolParser.class);

    private static final String XML_TAG_LISTS = "lists";
    private static final String XML_TAG_TASKS = "tasks";
    private static final String XML_ATTR_NAME = "name";
    private static final String XML_ATTR_EXTENSION_ID = "extensionId";
    private static final String XML_ATTR_ID = "id";

    private final Element protocol;

    PluginProtocolParser() throws IOException, JDOMException {
        this.protocol = parsePluginProtocol();
    }

    private Element parsePluginProtocol() throws IOException, JDOMException {
        File protocolFile = new File(ConfigProvider.getInstance().getPluginProtocol());
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

    <T> List<ProtocolTask> getDeclaredTasks(Class<T> type) {
        if (protocol == null) return Collections.emptyList();

        Element tasksRoot = protocol.getChild(XML_TAG_TASKS);
        if (tasksRoot == null) {
            LOGGER.warn("No tasks root specified in protocol. Please add <tasks></tasks>");
            return Collections.emptyList();
        }

        Element tasksForType = tasksRoot.getChild(type.getSimpleName());
        if (tasksForType == null) return Collections.emptyList();

        return tasksForType.getChildren()
                .stream()
                .map(t -> new ProtocolTask(t.getAttributeValue(XML_ATTR_ID), getProtocolExtensions(t)))
                .collect(Collectors.toList());
    }

    <T> List<ProtocolExtension> getProtocolExtensions(Class<T> type) {
        return getProtocolExtensions(protocol.getChild(XML_TAG_LISTS).getChild(type.getSimpleName()));
    }

    List<ProtocolExtension> getProtocolExtensions(Element root) {
        if (root == null) return Collections.emptyList();

        return root.getChildren()
                .stream()
                .map(this::getProtocolExtension)
                .collect(Collectors.toList());
    }

    private ProtocolExtension getProtocolExtension(Element pluginElement) {
        String name = pluginElement.getAttributeValue(XML_ATTR_NAME);
        String extensionId = pluginElement.getAttributeValue(XML_ATTR_EXTENSION_ID);
        return new ProtocolExtension(name, extensionId, pluginElement.clone());
    }
}
