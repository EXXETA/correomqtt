package org.correomqtt.core.plugin.spi;

import java.util.List;

public interface IncomingMessageHookDTO {

    boolean isEnableIncoming();

    List<String> getIncomingTopicFilter();
}
