package org.correomqtt.business.model;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonInclude;
import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Getter;
import lombok.NoArgsConstructor;
import lombok.Setter;
import org.correomqtt.core.model.ConnectionConfigDTO;
import org.correomqtt.core.model.ThemeSettingsDTO;

import java.util.List;

@Getter
@Setter
@AllArgsConstructor
@NoArgsConstructor
@Builder
@JsonInclude(JsonInclude.Include.NON_NULL)
@JsonIgnoreProperties(ignoreUnknown = true)
public class ConfigDTO {
    private List<ConnectionConfigDTO> connections;
    private ThemeSettingsDTO themesSettings;
    private SettingsDTO settings;
}
