package org.correomqtt.plugin.manager;

import lombok.Getter;
import lombok.RequiredArgsConstructor;
import org.jdom2.Element;

@Getter
@RequiredArgsConstructor
class ProtocolExtension {

    private final String pluginName;
    private final String extensionId;
    private final Element pluginConfig;
}
