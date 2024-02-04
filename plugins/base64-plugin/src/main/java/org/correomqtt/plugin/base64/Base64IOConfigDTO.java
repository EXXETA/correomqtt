package org.correomqtt.plugin.base64;

import lombok.AllArgsConstructor;
import lombok.Getter;
import lombok.NoArgsConstructor;
import lombok.Setter;

import java.util.List;

@Getter
@Setter
@NoArgsConstructor
@AllArgsConstructor
public class Base64IOConfigDTO {

    private boolean enableIncoming;

    private boolean enableOutgoing;

    private List<String> incomingTopicFilter;

    private List<String> outgoingTopicFilter;
}
