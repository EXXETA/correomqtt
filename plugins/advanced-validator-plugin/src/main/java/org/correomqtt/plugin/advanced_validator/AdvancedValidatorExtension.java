package org.correomqtt.plugin.advanced_validator;

import com.fasterxml.jackson.core.type.TypeReference;
import dagger.Component;
import javafx.scene.paint.Color;
import org.correomqtt.MainComponent;
import org.correomqtt.core.PluginScoped;
import org.correomqtt.core.CoreManager;
import org.correomqtt.core.model.HooksDTO;
import org.correomqtt.core.plugin.PluginManager;
import org.correomqtt.core.plugin.spi.ExtensionId;
import org.correomqtt.core.plugin.spi.MessageValidatorHook;
import org.correomqtt.gui.plugin.ExtensionComponent;
import org.correomqtt.gui.plugin.spi.ThemeProviderHook;
import org.correomqtt.gui.theme.IconMode;
import org.correomqtt.gui.theme.ThemeProvider;
import org.pf4j.Extension;

import org.correomqtt.core.cdi.Inject;
import java.util.List;
import java.util.stream.Stream;

@Extension(points = {MessageValidatorHook.class})
@ExtensionId("advanced")
public class AdvancedValidatorExtension implements ThemeProvider, ThemeProviderHook { //implements MessageValidatorHook<AdvancedValidatorConfig> {

    private final PluginManager pluginManager;
    private AdvancedValidatorConfig initialConfig;

    @PluginScoped
    @Component(dependencies = MainComponent.class)
    public interface Factory extends ExtensionComponent<AdvancedValidatorExtension> {
    }


    @Inject
    public AdvancedValidatorExtension(CoreManager coreManager) {
        this.pluginManager = coreManager.getPluginManager();
    }
/*
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
*/
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

    @Override
    public String getName() {
        return "doo" + (pluginManager != null ? "YES" : "NO");
    }

    @Override
    public String getCss() {
        return null;
    }

    @Override
    public IconMode getIconMode() {
        return null;
    }

    @Override
    public Color getBackgroundColor() {
        return null;
    }
}
