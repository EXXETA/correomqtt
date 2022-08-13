package org.correomqtt.plugin.transformer;

import com.github.zafarkhaja.semver.Version;
import org.correomqtt.plugin.model.PluginInfoDTO;
import org.pf4j.PluginDescriptor;
import org.pf4j.PluginWrapper;
import org.pf4j.update.PluginInfo;

import java.util.Comparator;

public class PluginInfoTransformer {

    public static PluginInfoDTO wrapperToDTO(PluginWrapper pluginWrapper, String installedVersion, boolean disabled){

        PluginDescriptor descriptor = pluginWrapper.getDescriptor();

        return PluginInfoDTO.builder()
                .id(pluginWrapper.getPluginId())
                .name(descriptor.getPluginId())
                .description(descriptor.getPluginDescription())
                .projectUrl(null)
                .provider(descriptor.getProvider())
                .repositoryId(null)
                .installableVersion(null)
                .installedVersion(installedVersion)
                .licence(descriptor.getLicense())
                .path(pluginWrapper.getPluginPath())
                .disabled(disabled)
                .build();
    }

    public static PluginInfoDTO pf4jToDTO(PluginInfo pluginInfo, String installedVersion, boolean disabled) {

        PluginInfo.PluginRelease installableRelease = pluginInfo.releases.stream()
                .min(Comparator.comparing(r -> Version.valueOf(r.version)))
                .orElse(null);

        String installableVersion = null;
        if(installableRelease != null){
            installableVersion = installableRelease.version;
        }

        return PluginInfoDTO.builder()
                .id(pluginInfo.id)
                .name(pluginInfo.name)
                .description(pluginInfo.description)
                .projectUrl(pluginInfo.projectUrl)
                .provider(pluginInfo.provider)
                .repositoryId(pluginInfo.getRepositoryId())
                .installableVersion(installableVersion)
                .installedVersion(installedVersion)
                .licence(null)
                .disabled(disabled)
                .path(null)
                .build();
    }
}
