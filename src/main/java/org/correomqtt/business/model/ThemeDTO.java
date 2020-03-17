package com.exxeta.correomqtt.business.model;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonInclude;
import lombok.Builder;
import lombok.Getter;
import lombok.NoArgsConstructor;
import lombok.Setter;

@Getter
@Setter
@Builder
@NoArgsConstructor
@JsonInclude(JsonInclude.Include.NON_NULL)
@JsonIgnoreProperties(ignoreUnknown = true)
public class ThemeDTO {

    private String name;
    private String file;
    private String iconMode;

    public ThemeDTO(String name, String file, String iconMode) {
        this.name = name;
        this.file = file;
        this.iconMode = iconMode;
    }
}
