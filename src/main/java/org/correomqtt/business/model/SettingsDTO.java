package org.correomqtt.business.model;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonInclude;
import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Getter;
import lombok.NoArgsConstructor;
import lombok.Setter;

import java.util.HashMap;
import java.util.Locale;
import java.util.Map;

@Getter
@Setter
@AllArgsConstructor
@NoArgsConstructor
@Builder
@JsonInclude(JsonInclude.Include.NON_NULL)
@JsonIgnoreProperties(ignoreUnknown = true)
public class SettingsDTO {

    private boolean useRegexForSearch;
    private boolean useIgnoreCase;
    private Locale savedLocale = null;
    private Locale currentLocale = null;
    private boolean searchUpdates;
    private Map<String, String> customPluginRepositories = new HashMap<>();
    private boolean firstStart = true;
    private String keyringIdentifier;
    @Builder.Default
    private GlobalUISettings globalUISettings = null;
    @Builder.Default
    private String configCreatedWithCorreoVersion = null;

    public boolean isUseRegexForSearch() {
        return useRegexForSearch;
    }

    public void setUseRegexForSearch(boolean useRegexForSearch) {
        this.useRegexForSearch = useRegexForSearch;
    }

    public boolean isUseIgnoreCase() {
        return useIgnoreCase;
    }

    public void setUseIgnoreCase(boolean useIgnoreCase) {
        this.useIgnoreCase = useIgnoreCase;
    }

    public Locale getSavedLocale() {
        return savedLocale;
    }

    public void setSavedLocale(Locale savedLocale) {
        this.savedLocale = savedLocale;
    }

    public GlobalUISettings getGlobalUISettings() {
        return globalUISettings;
    }

    public void setGlobalUISettings(GlobalUISettings globalUISettings) {
        this.globalUISettings = globalUISettings;
    }
}
