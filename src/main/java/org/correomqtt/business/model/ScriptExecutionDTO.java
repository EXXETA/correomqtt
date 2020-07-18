package org.correomqtt.business.model;

import lombok.*;
import java.io.OutputStream;

@Getter
@Setter
@Builder
@NoArgsConstructor
@AllArgsConstructor
public class ScriptExecutionDTO {

    private String connectionId;

    private String jsCode;

    private OutputStream out;
}
