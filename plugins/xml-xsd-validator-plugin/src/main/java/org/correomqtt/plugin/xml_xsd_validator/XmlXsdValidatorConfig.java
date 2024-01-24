package org.correomqtt.plugin.xml_xsd_validator;

import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Data;
import lombok.NoArgsConstructor;

@Data
@NoArgsConstructor
@AllArgsConstructor
@Builder
public class XmlXsdValidatorConfig {

    private String schema;
}
