package org.correomqtt.business.scripting;

import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Getter;
import lombok.NoArgsConstructor;
import lombok.Setter;

import java.io.PipedInputStream;
import java.time.LocalDateTime;
import java.time.temporal.ChronoUnit;
import java.util.UUID;

@Getter
@Setter
@Builder
@NoArgsConstructor
@AllArgsConstructor
public class ExecutionDTO {

    @Builder.Default
    private String executionId = UUID.randomUUID().toString();
    private String connectionId;
    private ScriptFileDTO scriptFile;
    private String jsCode;
    private PipedInputStream in;
    private ScriptExecutionError error;
    private LocalDateTime startTime;
    private Long executionTime;
    @Builder.Default
    private boolean cancelled = false;

    public void updateExecutionTime() {
        executionTime = getStartTime().until(LocalDateTime.now(), ChronoUnit.MILLIS);
    }
}
