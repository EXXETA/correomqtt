package org.correomqtt.plugin.base64;

import lombok.AllArgsConstructor;
import lombok.Getter;
import lombok.NoArgsConstructor;
import lombok.Setter;
import org.correomqtt.core.plugin.spi.OutgoingMessageHookDTO;

import java.util.List;

@Getter
@Setter
@NoArgsConstructor
@AllArgsConstructor
public class Base64OutgoingHookDTO implements OutgoingMessageHookDTO {

    private boolean enabled;

    private List<String> topicFilter;
}
