package org.correomqtt.plugin.base64;

import lombok.AllArgsConstructor;
import lombok.Getter;
import lombok.NoArgsConstructor;
import lombok.Setter;
import org.correomqtt.core.plugin.spi.IncomingMessageHookDTO;

import java.util.List;

@Getter
@Setter
@NoArgsConstructor
@AllArgsConstructor
public class Base64IncomingHookDTO implements IncomingMessageHookDTO {

    private boolean enabled;

    private List<String> topicFilter;
}
