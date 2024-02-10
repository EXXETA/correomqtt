package org.correomqtt.core.plugin;

import org.correomqtt.di.DefaultBean;
import org.correomqtt.core.plugin.spi.MessageValidatorHook;

import org.correomqtt.di.Inject;
import java.util.List;

@DefaultBean
public class MessageValidator {

    private final PluginManager pluginManager;

    @Inject
    public MessageValidator(PluginManager pluginManager) {
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
