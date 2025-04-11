package org.correomqtt.core.plugin.spi;

import java.util.List;

public interface OutgoingMessageHookDTO {

    boolean isEnabled();

    List<String> getTopicFilter();
}
