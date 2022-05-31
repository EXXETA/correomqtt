package org.correomqtt.plugin.manager;

import org.correomqtt.plugin.spi.MessageValidatorHook;

import java.util.List;

public class MessageValidator {

    private MessageValidator(){
        // private constructor
    }

    public static MessageValidatorHook.Validation validateMessage(String topic, String payload) {
        List<MessageValidatorHook<?>> validators = PluginManager.getInstance().getMessageValidators(topic);

        MessageValidatorHook.Validation validation = null;
        for (MessageValidatorHook<?> validator : validators) {
            validation = validator.isMessageValid(payload);
            if (validation.isValid()) break;
        }
        return validation;
    }
}
