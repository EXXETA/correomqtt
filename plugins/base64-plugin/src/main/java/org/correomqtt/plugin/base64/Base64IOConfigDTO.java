package org.correomqtt.plugin.base64;

import lombok.AllArgsConstructor;
import lombok.Getter;
import lombok.NoArgsConstructor;
import lombok.Setter;

@Getter
@Setter
@NoArgsConstructor
@AllArgsConstructor
public class Base64IOConfigDTO {

    private Base64OutgoingHookDTO outgoing;

    private Base64IncomingHookDTO incoming;
}
