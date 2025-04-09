package org.correomqtt.gui.transformer;

import org.correomqtt.core.model.SubscriptionDTO;
import org.correomqtt.gui.model.SubscriptionPropertiesDTO;

public class SubscriptionTransformer {

    private SubscriptionTransformer() {
        //private constructor
    }

    public static SubscriptionDTO propsToDTO(SubscriptionPropertiesDTO props) {
        return SubscriptionDTO.builder()
                              .topic(props.getTopic())
                              .qos(props.getQos())
                              .hidden(props.isHidden())
                              .build();
    }

    public static SubscriptionPropertiesDTO dtoToProps(SubscriptionDTO dto) {
        return SubscriptionPropertiesDTO.builder()
                                        .topic(dto.getTopic())
                                        .qos(dto.getQos())
                                        .hidden((dto.isHidden()))
                                        .build();
    }
}
