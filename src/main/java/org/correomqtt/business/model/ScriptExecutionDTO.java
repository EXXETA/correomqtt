package org.correomqtt.business.model;

import lombok.*;
import java.io.OutputStream;
import java.util.UUID;

@Getter
@Setter
@Builder
@NoArgsConstructor
@AllArgsConstructor
public class ScriptExecutionDTO {

    @Builder.Default
    private String executionId = UUID.randomUUID().toString();
    private String connectionId;
    private String jsCode;
}
