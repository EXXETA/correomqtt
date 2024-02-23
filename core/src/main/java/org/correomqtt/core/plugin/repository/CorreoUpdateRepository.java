package org.correomqtt.core.plugin.repository;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.correomqtt.core.utils.VersionUtils;
import org.pf4j.update.FileDownloader;
import org.pf4j.update.FileVerifier;
import org.pf4j.update.PluginInfo;
import org.pf4j.update.SimpleFileDownloader;
import org.pf4j.update.UpdateRepository;
import org.pf4j.update.verifier.CompoundVerifier;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;
import java.net.MalformedURLException;
import java.net.URL;
import java.util.ArrayList;
import java.util.Collections;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

public class CorreoUpdateRepository implements UpdateRepository {

    private static final Logger log = LoggerFactory.getLogger(CorreoUpdateRepository.class);

    private final String id;
    private final URL url;
    private final String originalUrl;

    private Map<String, PluginInfo> plugins;

    /**
     * @param id  the repository id
     * @param url the repository url
     */
    public CorreoUpdateRepository(String id, String url) throws MalformedURLException {
        this.id = id;
        this.url = new URL(url);
        this.originalUrl = url;
    }

    @Override
    public String getId() {
        return id;
    }

    @Override
    public URL getUrl() {
        return url;
    }

    @Override
    public Map<String, PluginInfo> getPlugins() {
        if (plugins == null) {
            initPlugins();
        }
        return plugins;
    }

    @Override
    public PluginInfo getPlugin(String id) {
        return getPlugins().get(id);
    }

    private void initPlugins() {
        RepoPluginInfoDTO[] items;
        try {
            log.info("Read plugins of '{}' repository from '{}'", id, originalUrl);
            items = new ObjectMapper().readValue(url, RepoPluginInfoDTO[].class);
        } catch (IOException e) {
            //TODO event for UI
            log.error("Unable to read plugin repository '{}'", originalUrl, e);
            plugins = Collections.emptyMap();
            return;
        }
        plugins = new HashMap<>();
        for (RepoPluginInfoDTO p : items) {
            List<RepoPluginInfoDTO.PluginRelease> releases = new ArrayList<>();
            for (RepoPluginInfoDTO.PluginRelease r : p.getReleases()) {
                if (isPluginCompatible(r)) {
                    if (r.getDate().getTime() == 0) {
                        log.warn("Illegal release date when parsing {}@{}, setting to epoch", p.getId(), r.getVersion());
                    }
                    releases.add(r);
                }
            }
            // Skip if plugin has no compatible releases
            if (releases.isEmpty()) {
                log.info("Plugin {} is not compatible to this CorreoMQTT version.", p.getName());
                continue;
            }
            p.setRepositoryId(getId());
            p.setReleases(releases);
            plugins.put(p.getId(), p.transformToPf4jInfo());
        }
        log.info("Found {} plugins in repository '{}' compatible with version {}",
                plugins.size(), originalUrl, VersionUtils.getVersion());
    }

    private boolean isPluginCompatible(RepoPluginInfoDTO.PluginRelease release) {
        List<String> compatibleCorreoVersions = release.getCompatibleCorreoVersions();
        return compatibleCorreoVersions != null &&
                release.getCompatibleCorreoVersions().stream()
                        .map(v -> VersionUtils.getMajorMinor(v).equals(VersionUtils.getMajorMinor(VersionUtils.getVersion())))
                        .filter(b -> b)
                        .findAny()
                        .orElse(false);
    }

    /**
     * Causes {@code plugins.json} to be read again to look for new updates from repositories.
     */
    @Override
    public void refresh() {
        plugins = null;
    }

    @Override
    public FileDownloader getFileDownloader() {
        return new SimpleFileDownloader();
    }

    /**
     * Gets a file verifier to execute on the downloaded file for it to be claimed valid.
     * May be a CompoundVerifier in order to chain several verifiers.
     *
     * @return list of {@link FileVerifier}s
     */
    @Override
    public FileVerifier getFileVerifier() {
        return new CompoundVerifier();
    }
}
