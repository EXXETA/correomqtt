package org.correomqtt.plugin.contains_string_validator;

import lombok.extern.slf4j.Slf4j;
import org.correomqtt.core.plugin.spi.MessageValidatorHook;

@Slf4j
public abstract class ContainsStringValidator implements MessageValidatorHook<ContainsStringValidatorConfig> {

    protected String text;

    @Override
    public Class<ContainsStringValidatorConfig> getConfigClass(){
        return ContainsStringValidatorConfig.class;
    }

    @Override
    public void onConfigReceived(ContainsStringValidatorConfig config) {
        text = config.getText();
    }
}
