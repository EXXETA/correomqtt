package org.correomqtt.core.plugin;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.correomqtt.core.fileprovider.PluginConfigProvider;
import org.correomqtt.core.model.HooksDTO;
import org.correomqtt.core.plugin.model.PluginInfoDTO;
import org.correomqtt.core.plugin.repository.BundledPluginList;
import org.correomqtt.core.plugin.repository.CorreoUpdateRepository;
import org.correomqtt.core.plugin.spi.BaseExtensionPoint;
import org.correomqtt.core.plugin.spi.ExtensionId;
import org.correomqtt.core.plugin.spi.IncomingMessageHook;
import org.correomqtt.core.plugin.spi.MessageValidatorHook;
import org.correomqtt.core.plugin.spi.OutgoingMessageHook;
import org.correomqtt.core.plugin.transformer.PluginInfoTransformer;
import org.correomqtt.core.settings.SettingsManager;
import org.correomqtt.core.utils.VendorConstants;
import org.correomqtt.core.utils.VersionUtils;
import org.correomqtt.di.Inject;
import org.correomqtt.di.SingletonBean;
import org.pf4j.DefaultExtensionFactory;
import org.pf4j.ExtensionFactory;
import org.pf4j.JarPluginManager;
import org.pf4j.PluginState;
import org.pf4j.PluginWrapper;
import org.pf4j.update.UpdateManager;
import org.pf4j.update.UpdateRepository;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;
import java.lang.reflect.ParameterizedType;
import java.lang.reflect.Type;
import java.net.HttpURLConnection;
import java.net.MalformedURLException;
import java.net.ProtocolException;
import java.net.URL;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Collection;
import java.util.Comparator;
import java.util.List;
import java.util.Objects;
import java.util.Optional;
import java.util.stream.Collectors;

@SingletonBean
public class PluginManager extends JarPluginManager {

    private static final Logger LOGGER = LoggerFactory.getLogger(PluginManager.class);
    public static final String DEFAULT_REPO_ID = "default";
    private final SettingsManager settings;
    private final PluginConfigProvider pluginConfigProvider;
    private BundledPluginList.BundledPlugins bundledPlugins;

    @Inject
    public PluginManager(SettingsManager settings,
                         PluginConfigProvider pluginConfigProvider) {
        super(Path.of(pluginConfigProvider.getPluginPath()));
        this.settings = settings;
        this.pluginConfigProvider = pluginConfigProvider;
    }

    @Override
    protected ExtensionFactory createExtensionFactory() {
        return Objects.requireNonNullElseGet(extensionFactory, DefaultExtensionFactory::new);
    }

    private String getInstalledVersion(String pluginId) {
        PluginWrapper installedPlugin = this.getPlugin(pluginId);
        if (installedPlugin == null) {
            return null;
        }
        return installedPlugin.getDescriptor().getVersion();
    }

    public List<PluginInfoDTO> getInstalledPlugins() {
        return this.getPlugins().stream()
                .map(wrapper -> PluginInfoTransformer.wrapperToDTO(wrapper, wrapper.getDescriptor().getVersion(),
                        isPluginDisabled(wrapper.getPluginId()),
                        isPluginBundled(wrapper.getPluginId())))
                .sorted(Comparator.comparing(PluginInfoDTO::getName))
                .toList();
    }

    public BundledPluginList.BundledPlugins getBundledPlugins() {
        if (bundledPlugins != null) {
            return bundledPlugins;
        }
        if (settings.getSettings().isInstallBundledPlugins()) {
            String bundledPluginUrl = settings.getSettings().getBundledPluginsUrl();
            if (bundledPluginUrl == null) {
                bundledPluginUrl = VendorConstants.getBundledPluginsUrl();
            }

            if (bundledPluginUrl.contains("{version}")) {
                String latestBundled = bundledPluginUrl.replace("{version}", "latest");
                if (checkUrl(latestBundled)) {
                    bundledPluginUrl = latestBundled;
                } else {
                    String versionBundled = bundledPluginUrl.replace("{version}", "v" + VersionUtils.getVersion());
                    if (checkUrl(versionBundled)) {
                        bundledPluginUrl = versionBundled;
                    }
                }
            }

            try {
                LOGGER.info("Read bundled plugins '{}'", bundledPluginUrl);
                BundledPluginList bundledPluginList = new ObjectMapper().readValue(new URL(bundledPluginUrl), BundledPluginList.class);
                BundledPluginList.BundledPlugins bundledPluginsByVersion = bundledPluginList.getVersions().get(VersionUtils.getVersion().trim());
                if (bundledPluginsByVersion == null) {
                    return BundledPluginList.BundledPlugins.builder().build();
                }
                bundledPlugins = bundledPluginsByVersion;
                return bundledPluginsByVersion;
            } catch (IOException e) {
                LOGGER.warn("Unable to load bundled plugin list from {}.", bundledPluginUrl);
                return BundledPluginList.BundledPlugins.builder().build();
            }
        } else {
            LOGGER.info("Do not install bundled plugins.");
            return BundledPluginList.BundledPlugins.builder().build();
        }
    }

    private boolean isPluginBundled(String pluginId) {
        return getBundledPlugins().getInstall().stream().anyMatch(p -> p.equals(pluginId));
    }

    public List<PluginInfoDTO> getAllPluginsAvailableFromRepos() {
        return getUpdateManager().getPlugins().stream()
                .map(info -> PluginInfoTransformer.pf4jToDTO(info, getInstalledVersion(info.id), isPluginDisabled(info.id)))
                .sorted(Comparator.comparing(PluginInfoDTO::getName))
                .toList();
    }

    private boolean checkUrl(String urlString) {
        try {
            URL url = new URL(urlString);
            HttpURLConnection huc = (HttpURLConnection) url.openConnection();
            huc.setRequestMethod("HEAD");
            return (huc.getResponseCode() == 200 || huc.getResponseCode() == 302);
        } catch (IOException e) {
            return false;
        }
    }

    public UpdateManager getUpdateManager() {
        List<UpdateRepository> repos = new ArrayList<>();
        if (settings.getSettings().isSearchUpdates()) {
            if (settings.getSettings().isUseDefaultRepo()) {
                String defaultRepo = VendorConstants.getDefaultRepoUrl();
                if (defaultRepo.contains("{version}")) {
                    String latestRepo = defaultRepo.replace("{version}", "latest");
                    if (checkUrl(latestRepo)) {
                        defaultRepo = latestRepo;
                    } else {
                        String versionRepo = defaultRepo.replace("{version}", "v" + VersionUtils.getVersion());
                        if (checkUrl(versionRepo)) {
                            defaultRepo = versionRepo;
                        }
                    }
                }
                try {
                    repos.add(new CorreoUpdateRepository(DEFAULT_REPO_ID, defaultRepo));
                } catch (MalformedURLException e) {
                    LOGGER.error("Invalid url for repo {} with url {}", DEFAULT_REPO_ID, defaultRepo);
                }
            }
            settings.getSettings().getPluginRepositories().forEach((id, url) -> {
                try {
                    repos.add(new CorreoUpdateRepository(id, url));
                } catch (MalformedURLException e) {
                    LOGGER.error("Invalid url for repo {} with url {}", id, url);
                }
            });
        }
        return new UpdateManager(this, repos);
    }

    public List<? extends OutgoingMessageHook<?>> getOutgoingMessageHooks() {
        return pluginConfigProvider.getOutgoingMessageHooks()
                .stream()
                .map(extensionDefinition -> {
                    OutgoingMessageHook<?> extension = getExtensionById(OutgoingMessageHook.class,
                            extensionDefinition.getPluginId(),
                            extensionDefinition.getId());
                    if (extension == null) {
                        LOGGER.warn("Extension for Outgoing Message Hook with id {} from plugin {} not found.", extensionDefinition.getId(), extensionDefinition.getPluginId());
                        return null;
                    }
                    enrichExtensionWithConfig(extension, extensionDefinition.getConfig());
                    return extension;
                })
                .filter(Objects::nonNull)
                .toList();
    }

    public List<? extends IncomingMessageHook<?>> getIncomingMessageHooks() {
        return pluginConfigProvider.getIncomingMessageHooks()
                .stream()
                .map(extensionDefinition -> {
                    IncomingMessageHook<?> extension = getExtensionById(IncomingMessageHook.class,
                            extensionDefinition.getPluginId(),
                            extensionDefinition.getId());
                    if (extension == null) {
                        LOGGER.warn("Extension for Incoming Message Hook with id {} from plugin {} not found.", extensionDefinition.getId(), extensionDefinition.getPluginId());
                        return null;
                    }
                    enrichExtensionWithConfig(extension, extensionDefinition.getConfig());
                    return extension;
                })
                .filter(Objects::nonNull)
                .toList();
    }

    public List<MessageValidatorHook<?>> getMessageValidators(String topic) {
        return pluginConfigProvider.getMessageValidators()
                .stream()
                .filter(validatorDefinition -> validatorDefinition.getTopic().equals(topic))
                .map(validatorDefinition -> validatorDefinition.getExtensions().stream()
                        .map(extensionDefinition -> {
                            String pluginId = extensionDefinition.getPluginId();
                            String extensionId = extensionDefinition.getId();
                            MessageValidatorHook<?> extension = getExtensionById(MessageValidatorHook.class, pluginId, extensionId);
                            if (extension == null) {
                                LOGGER.warn("Plugin extension {}:{} in messageValidators is configured, but does not exist.", pluginId, extensionId);
                                return null;
                            }
                            enrichExtensionWithConfig(extension, extensionDefinition.getConfig());
                            return extension;
                        })
                        .toList())
                .flatMap(Collection::stream)
                .collect(Collectors.toList());
    }

    public <T> void enrichExtensionWithConfig(BaseExtensionPoint<T> extension, JsonNode configNode) {
        try {
            Class<T> configClass = extension.getConfigClass();
            if (configClass != null) {
                extension.onConfigReceived(new ObjectMapper().treeToValue(configNode, configClass));
            }
        } catch (JsonProcessingException e) {
            LOGGER.error("Exception parsing plugin configuration object for {}", extension.getConfigClass());
        }
    }

    public <P extends BaseExtensionPoint<T>, T> P getExtensionByIdWithConfig(Class<P> type, String pluginId, String extensionId, T config) {
        P extension = getExtensionById(type, pluginId, extensionId);
        if (extension != null) {
            extension.onConfigReceived(config);
        }
        return extension;
    }

    public <P extends BaseExtensionPoint<?>> P getExtensionById(Class<P> type, String pluginId, String extensionId) {
        return super.getExtensions(type, pluginId)
                .stream()
                .filter(e -> isExtensionIdResolved(e, extensionId))
                .findFirst()
                .orElseGet(() -> {
                    logInvalidPluginDeclaration(type, pluginId, extensionId);
                    return null;
                });
    }

    @Override
    public <T> List<T> getExtensions(Class<T> type, String pluginId) {
        return getExtensions(type);
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
            PluginWrapper pluginWrapper = getPlugin(pluginId);
            if (pluginWrapper != null && getPlugin(pluginId).getPluginState().equals(PluginState.STARTED)) {
                LOGGER.warn("Plugin {} declared for {} has no valid extension", pluginId, type.getSimpleName());
            } else {
                LOGGER.warn("Plugin {} declared for {} is not started", pluginId, type.getSimpleName());
            }
        }
    }

    @Override
    public void unloadPlugins() {
        LOGGER.debug("Unload Plugins");
        List<String> pluginIds = resolvedPlugins.stream().map(PluginWrapper::getPluginId).toList();
        for (String pluginId : pluginIds) {
            LOGGER.debug("Unload Plugin \"{}\"", pluginId);
            unloadPlugin(pluginId);
        }
    }

    public <P extends BaseExtensionPoint<T>, T> P getExtensionByDefinition(Class<P> clazz, HooksDTO.Extension extensionDefinition) {
        P extension = getExtensionById(clazz, extensionDefinition.getPluginId(), extensionDefinition.getId());
        enrichExtensionWithConfig(extension, extensionDefinition.getConfig());
        return extension;
    }

    @SuppressWarnings("unchecked")
    public <P extends BaseExtensionPoint<T>, T> P getExtensionByDefinition(TypeReference<P> typeReference, HooksDTO.Extension extensionDefinition) {
        Type type = typeReference.getType();
        // https://stackoverflow.com/a/28615143
        Class<P> clazz = (Class<P>) (type instanceof ParameterizedType parameterizedType ?
                parameterizedType.getRawType() :
                type);
        return getExtensionByDefinition(clazz, extensionDefinition);
    }

    public void setExtensionFactory(ExtensionFactory extensionFactory) {
        this.extensionFactory = extensionFactory;
    }
}
