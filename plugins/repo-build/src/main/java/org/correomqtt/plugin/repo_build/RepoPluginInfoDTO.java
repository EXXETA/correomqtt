package org.correomqtt.plugin.repo_build;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Getter;
import lombok.NoArgsConstructor;
import lombok.Setter;

import java.io.Serializable;
import java.time.LocalDate;
import java.util.List;

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

    @Getter
    @Setter
    @NoArgsConstructor
    @AllArgsConstructor
    @Builder
    @JsonIgnoreProperties(ignoreUnknown = true)
    public static class PluginRelease implements Serializable {

        private String version;
        @RepoDateFormatter
        private LocalDate date;
        private String requires;
        private String url;
        private String sha512sum;
        private List<String> compatibleCorreoVersions;

    }
}
