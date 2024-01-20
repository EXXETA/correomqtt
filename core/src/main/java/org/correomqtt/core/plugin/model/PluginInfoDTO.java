package org.correomqtt.core.plugin.model;

import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Getter;
import lombok.NoArgsConstructor;
import lombok.Setter;

import java.nio.file.Path;

@Getter
@Setter
@NoArgsConstructor
@AllArgsConstructor
@Builder
public class PluginInfoDTO {

    private String id;
    private String name;
    private String description;
    private String provider;
    private String projectUrl;
    private String repositoryId;
    private String installedVersion;
    private String installableVersion;
    private String licence;
    private Path path;
    private boolean disabled;
    private boolean bundled;

    private boolean upgradeable;
}
