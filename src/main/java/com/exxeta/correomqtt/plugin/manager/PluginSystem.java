package com.exxeta.correomqtt.plugin.manager;

import com.exxeta.correomqtt.business.services.ConfigService;
import org.jdom2.JDOMException;
import org.pf4j.*;

import java.io.IOException;
import java.nio.file.Path;
import java.util.List;

public class PluginSystem extends DefaultPluginManager {

    private static PluginSystem instance;

    private PluginProtocolParser pluginProtocolParser;

    private PluginSystem() {
        // private constructor
        super(Path.of(ConfigService.getInstance().getPluginJarPath()));
        try {
            pluginProtocolParser = new PluginProtocolParser(this);
        } catch (IOException | JDOMException e) {
            e.printStackTrace();
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
        List<T> extensions = pluginProtocolParser.getDeclaredExtensions(type);
        if (extensions.isEmpty()) return super.getExtensions(type);
        else return extensions;
    }

    public <T> List<PluginProtocolTask<T>> getTasks(Class<T> type) {
        return pluginProtocolParser.getDeclaredTasks(type);
    }
}
