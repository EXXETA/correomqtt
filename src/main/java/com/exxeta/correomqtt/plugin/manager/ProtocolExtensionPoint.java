package com.exxeta.correomqtt.plugin.manager;

import lombok.Getter;
import lombok.RequiredArgsConstructor;
import org.jdom2.Element;

@Getter
@RequiredArgsConstructor
class ProtocolExtensionPoint {

    private final String pluginName;
    private final String extensionId;
    private final Element pluginConfig;
}
