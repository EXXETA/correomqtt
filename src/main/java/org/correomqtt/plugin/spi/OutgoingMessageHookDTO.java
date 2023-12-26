package org.correomqtt.plugin.spi;

import java.util.List;

public interface OutgoingMessageHookDTO {

    boolean isEnableOutgoing();

    List<String> getOutgoingTopicFilter();


}
