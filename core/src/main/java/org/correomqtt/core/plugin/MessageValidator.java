package org.correomqtt.core.plugin;

import org.correomqtt.core.plugin.spi.MessageValidatorHook;

import javax.inject.Inject;
import java.util.List;

public class MessageValidator {

    private final PluginManager pluginManager;

    @Inject
    MessageValidator(PluginManager pluginManager) {
        this.pluginManager = pluginManager;
    }

    public MessageValidatorHook.Validation validateMessage(String topic, String payload) {
        List<MessageValidatorHook<?>> validators = pluginManager.getMessageValidators(topic);

        MessageValidatorHook.Validation validation = null;
        for (MessageValidatorHook<?> validator : validators) {
            validation = validator.isMessageValid(payload);
            if (validation.isValid()) break;
        }
        return validation;
    }
}
