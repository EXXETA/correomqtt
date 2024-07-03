package org.correomqtt.plugin.contains_string_validator;

import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Data;
import lombok.NoArgsConstructor;

@Data
@NoArgsConstructor
@AllArgsConstructor
@Builder
public class ContainsStringValidatorConfig {

    private String text;
}
