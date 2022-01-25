package org.correomqtt.plugin.manager;

import org.correomqtt.plugin.spi.MessageValidatorHook;
import com.hivemq.client.mqtt.datatypes.MqttTopic;
import com.hivemq.client.mqtt.datatypes.MqttTopicFilter;

import java.util.Optional;

public class MessageValidator {

    private MessageValidator(){
        // private constructor
    }

    public static MessageValidatorHook.Validation validateMessage(String topic, String payload) {
        Optional<Task<MessageValidatorHook>> validatorTaskOptional = PluginManager.getInstance()
                                                                                  .getTasks(MessageValidatorHook.class)
                                                                                  .stream()
                                                                                  .filter(t -> MqttTopicFilter.of(t.getId()).matches(MqttTopic.of(topic)))
                                                                                  .findFirst();
        if (validatorTaskOptional.isEmpty()) return null;

        MessageValidatorHook.Validation validation = null;
        for (MessageValidatorHook v : validatorTaskOptional.get().getTasks()) {
            validation = v.isMessageValid(payload);
            if (validation.isValid()) break;
        }
        return validation;
    }
}
