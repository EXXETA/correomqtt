package org.correomqtt.plugin.contains_string_validator;

import org.correomqtt.core.plugin.spi.ExtensionId;
import org.correomqtt.core.plugin.spi.MessageValidatorHook;
import org.pf4j.Extension;

@Extension(points = {MessageValidatorHook.class})
@ExtensionId("caseSensitive")
public class CaseSensitiveContainsStringValidator extends ContainsStringValidator {

    @Override
    public Validation isMessageValid(String payload) {
        String tooltip = "Contains '" + text + "' - case sensitive";
        return new Validation(payload.contains(text), tooltip);
    }
}
