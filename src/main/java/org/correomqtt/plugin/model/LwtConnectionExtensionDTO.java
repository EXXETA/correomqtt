package org.correomqtt.plugin.model;

import lombok.Getter;
import lombok.Setter;
import org.correomqtt.business.model.Lwt;
import org.correomqtt.business.model.Qos;
import org.correomqtt.gui.model.ConnectionPropertiesDTO;

import java.util.HashMap;

@Setter
@Getter
public class LwtConnectionExtensionDTO {

    private String id;
    private String name;
    private Lwt lwt;
    private String lwtTopic;
    private Qos lwtQoS;
    private boolean lwtRetained;
    private String lwtPayload;
    private HashMap<String, Object> customFields;

    public LwtConnectionExtensionDTO(ConnectionPropertiesDTO connectionPropertiesDTO) {
        this.id = connectionPropertiesDTO.getId();
        this.name = connectionPropertiesDTO.getName();
        this.lwt = connectionPropertiesDTO.getLwt();
        this.lwtTopic = connectionPropertiesDTO.getLwtTopic();
        this.lwtQoS = connectionPropertiesDTO.getLwtQos();
        this.lwtRetained = connectionPropertiesDTO.isLwtRetained();
        this.lwtPayload = connectionPropertiesDTO.getLwtPayload();
        this.customFields = new HashMap<>(connectionPropertiesDTO.getExtraProperties());
    }
}
