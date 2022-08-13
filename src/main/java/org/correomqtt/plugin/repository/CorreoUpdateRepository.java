package org.correomqtt.plugin.repository;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.github.zafarkhaja.semver.Version;
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
    private final Version apiLevel;
    private final String originalUrl;

    private Map<String, PluginInfo> plugins;

    /**
     * @param id  the repository id
     * @param url the repository url
     */
    public CorreoUpdateRepository(String id, String url, String apiLevel) throws MalformedURLException {
        this.id = id;
        this.url = new URL(url);
        this.originalUrl = url;
        this.apiLevel = Version.valueOf(apiLevel);
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
            log.debug("Read plugins of '{}' repository from '{}'", id, url);
            items = new ObjectMapper().readValue(url, RepoPluginInfoDTO[].class);
        } catch (IOException e) {
            //TODO event for UI
            log.error("Unable to read plugin repository '{}'", url, e);
            plugins = Collections.emptyMap();
            return;
        }

        plugins = new HashMap<>();
        for (RepoPluginInfoDTO p : items) {
            List<RepoPluginInfoDTO.PluginRelease> releases = new ArrayList<>();
            for (RepoPluginInfoDTO.PluginRelease r : p.getReleases()) {
                if (fitApiLevel(r)) {
                    if (r.getDate().getTime() == 0) {
                        log.warn("Illegal release date when parsing {}@{}, setting to epoch", p.getId(), r.getVersion());
                    }
                    releases.add(r);
                }
            }

            // Skip if plugin has no compatible releases
            if(releases.isEmpty()){
                continue;
            }

            p.setRepositoryId(getId());
            p.setReleases(releases);
            plugins.put(p.getId(), p.transformToPf4jInfo());
        }
        log.info("Found {} plugins in repository '{}' compatible with api level {}", plugins.size(), originalUrl, apiLevel);
    }

    private boolean fitApiLevel(RepoPluginInfoDTO.PluginRelease release) {
        Version releaseApiLevel = Version.valueOf(release.getPluginApiLevel());
        // ApiLevel of release is greater or equal than the supported one, but has the same major version.
        return releaseApiLevel.compareTo(apiLevel) >= 0 && releaseApiLevel.getMajorVersion() == apiLevel.getMajorVersion();
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
     * @return list of {@link FileVerifier}s
     */
    @Override
    public FileVerifier getFileVerifier() {
        return new CompoundVerifier();
    }

}
