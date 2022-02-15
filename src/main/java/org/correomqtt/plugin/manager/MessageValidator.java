package org.correomqtt.plugin.manager;

import org.correomqtt.plugin.spi.MessageValidatorHook;
import com.hivemq.client.mqtt.datatypes.MqttTopic;
import com.hivemq.client.mqtt.datatypes.MqttTopicFilter;

import java.util.List;
import java.util.Optional;

public class MessageValidator {

    private MessageValidator(){
        // private constructor
    }

    public static MessageValidatorHook.Validation validateMessage(String topic, String payload) {
        List<MessageValidatorHook> validators = PluginManager.getInstance().getMessageValidators(topic);

        MessageValidatorHook.Validation validation = null;
        for (MessageValidatorHook validator : validators) {
            validation = validator.isMessageValid(payload);
            if (validation.isValid()) break;
        }
        return validation;
    }
}
