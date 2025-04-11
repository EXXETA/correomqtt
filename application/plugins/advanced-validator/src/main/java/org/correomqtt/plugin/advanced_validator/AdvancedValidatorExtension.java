package org.correomqtt.plugin.advanced_validator;

import com.fasterxml.jackson.core.type.TypeReference;
import org.correomqtt.core.CoreManager;
import org.correomqtt.core.model.HooksDTO;
import org.correomqtt.core.plugin.PluginManager;
import org.correomqtt.core.plugin.spi.ExtensionId;
import org.correomqtt.core.plugin.spi.MessageValidatorHook;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import org.pf4j.Extension;

import java.util.List;
import java.util.stream.Stream;

@Extension(points = {MessageValidatorHook.class})
@ExtensionId("advanced")
@DefaultBean
public class AdvancedValidatorExtension implements MessageValidatorHook<AdvancedValidatorConfig> {

    private final PluginManager pluginManager;
    private AdvancedValidatorConfig initialConfig;

    @Inject
    public AdvancedValidatorExtension(CoreManager coreManager) {
        this.pluginManager = coreManager.getPluginManager();
    }

    @Override
    public void onConfigReceived(AdvancedValidatorConfig config) {
        this.initialConfig = config;
    }

    @Override
    public Validation isMessageValid(String message) {
        boolean result = isMessageValidAnd(initialConfig, message);
        if (result) {
            return new Validation(true, "All valid");
        }
        return new Validation(false, "Invalid");
    }

    private boolean isMessageValidAnd(AdvancedValidatorConfig config, String message) {
        return config.getAnd().stream().allMatch(and -> isMessageValidAnd(and, message)) &&
                config.getOr().stream().allMatch(or -> isMessageValidOr(or, message)) &&
                getValidatorStream(config.getExtensions()).allMatch(v -> v.isMessageValid(message).isValid());
    }

    private boolean isMessageValidOr(AdvancedValidatorConfig config, String message) {
        return config.getAnd().stream().anyMatch(and -> isMessageValidAnd(and, message)) &&
                config.getOr().stream().anyMatch(or -> isMessageValidOr(or, message)) &&
                getValidatorStream(config.getExtensions()).anyMatch(v -> v.isMessageValid(message).isValid());
    }

    private Stream<MessageValidatorHook<AdvancedValidatorConfig>> getValidatorStream(List<HooksDTO.Extension> extensions) {
        return extensions.stream().
                map(extensionDefinition -> pluginManager
                        .getExtensionByDefinition(new TypeReference<MessageValidatorHook<AdvancedValidatorConfig>>() {
                        }, extensionDefinition)
                );
    }
}
