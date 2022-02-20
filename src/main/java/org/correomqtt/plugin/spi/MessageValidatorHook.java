package org.correomqtt.plugin.spi;

import lombok.Getter;
import lombok.RequiredArgsConstructor;

public interface MessageValidatorHook<T> extends BaseExtensionPoint<T> {

    @Getter
    @RequiredArgsConstructor
    class Validation {
        private final boolean isValid;
        private final String tooltip;
    }

    Validation isMessageValid(String message);
}
