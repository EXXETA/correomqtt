package org.correomqtt.core.plugin.model;

import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Getter;
import lombok.Setter;
import org.correomqtt.core.model.Lwt;
import org.correomqtt.core.model.Qos;

import java.util.HashMap;

@Setter
@Getter
@Builder
@AllArgsConstructor
public class LwtConnectionExtensionDTO {

    private String id;
    private String name;
    private Lwt lwt;
    private String lwtTopic;
    private Qos lwtQoS;
    private boolean lwtRetained;
    private String lwtPayload;
    private HashMap<String, Object> customFields;

}
