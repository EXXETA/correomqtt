package org.correomqtt.plugins.systopic.model;

import org.correomqtt.core.model.MessageDTO;

public class SysTopicTransformer {

    private SysTopicTransformer() {
        //private constructor
    }

    public static SysTopicPropertiesDTO dtoToProps(MessageDTO messageDTO, SysTopic sysTopic) {
        return SysTopicPropertiesDTO.builder()
                                   .topic(messageDTO.getTopic())
                                   .payload(messageDTO.getPayload())
                                   .sysTopic(sysTopic)
                                   .dateTime(messageDTO.getDateTime())
                                   .build();
    }
}
