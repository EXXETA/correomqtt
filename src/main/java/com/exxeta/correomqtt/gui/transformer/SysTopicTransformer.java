package com.exxeta.correomqtt.gui.transformer;

import com.exxeta.correomqtt.business.model.MessageDTO;
import com.exxeta.correomqtt.business.model.SysTopic;
import com.exxeta.correomqtt.gui.model.SysTopicPropertiesDTO;

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
