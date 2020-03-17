package org.correomqtt.gui.transformer;

import org.correomqtt.business.model.MessageDTO;
import org.correomqtt.business.model.SysTopic;
import org.correomqtt.gui.model.SysTopicPropertiesDTO;

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
