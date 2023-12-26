package org.correomqtt.gui.transformer;

import org.correomqtt.gui.model.PluginInfoPropertiesDTO;
import org.correomqtt.plugin.model.PluginInfoDTO;

import java.util.List;
import java.util.stream.Collectors;

public class PluginTransformer {

    private PluginTransformer() {
        // private constructor
    }

    public static List<PluginInfoPropertiesDTO> dtoListToPropList(List<PluginInfoDTO> pluginInfoList) {
        return pluginInfoList.stream()
                .map(PluginTransformer::dtoToProps)
                .toList();
    }

    public static PluginInfoPropertiesDTO dtoToProps(PluginInfoDTO dto) {

        return PluginInfoPropertiesDTO.builder()
                .id(dto.getId())
                .name(dto.getName())
                .projectUrl(dto.getProjectUrl())
                .description(dto.getDescription())
                .repositoryId(dto.getRepositoryId())
                .installedVersion(dto.getInstalledVersion())
                .installableVersion(dto.getInstallableVersion())
                .license(dto.getLicence())
                .disabled(dto.isDisabled())
                .bundled(dto.isBundled())
                .upgradeable(dto.isUpgradeable())
                .path(dto.getPath())
                .build();
    }
}
