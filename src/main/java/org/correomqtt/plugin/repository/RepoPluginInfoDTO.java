package org.correomqtt.plugin.repository;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Getter;
import lombok.NoArgsConstructor;
import lombok.Setter;
import org.pf4j.update.PluginInfo;

import java.io.Serializable;
import java.util.Date;
import java.util.List;
import java.util.stream.Collectors;

@Getter
@Setter
@NoArgsConstructor
@AllArgsConstructor
@Builder
@JsonIgnoreProperties(ignoreUnknown = true)
public class RepoPluginInfoDTO {

    private String id;
    private String name;
    private String description;
    private String provider;
    private String projectUrl;
    private List<PluginRelease> releases;
    private String repositoryId;

    public PluginInfo transformToPf4jInfo() {
        PluginInfo info = new PluginInfo();
        info.id = id;
        info.name = name;
        info.description = description;
        info.provider = provider;
        info.projectUrl = projectUrl;
        info.setRepositoryId(repositoryId);
        info.releases = releases.stream()
                .map(r -> {
                    PluginInfo.PluginRelease release = new PluginInfo.PluginRelease();
                    release.version = r.version;
                    release.date = r.date;
                    release.requires = r.requires;
                    release.url = r.url;
                    release.sha512sum = r.sha512sum;
                    return release;
                })
                .toList();
        return info;
    }

    @Getter
    @Setter
    @NoArgsConstructor
    @AllArgsConstructor
    @Builder
    @JsonIgnoreProperties(ignoreUnknown = true)
    public static class PluginRelease implements Serializable {

        private String version;
        private Date date;
        private String requires;
        private String url;
        private String sha512sum;
        private List<String> compatibleCorreoVersions;

    }
}
