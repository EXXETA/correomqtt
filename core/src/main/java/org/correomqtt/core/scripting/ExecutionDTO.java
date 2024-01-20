package org.correomqtt.core.scripting;

import ch.qos.logback.classic.Logger;
import com.fasterxml.jackson.annotation.JsonFormat;
import com.fasterxml.jackson.annotation.JsonIgnore;
import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Getter;
import lombok.NoArgsConstructor;
import lombok.Setter;

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
    @JsonIgnore
    private ScriptFileDTO scriptFile;
    @JsonIgnore
    private String jsCode;
    @JsonIgnore
    private Logger logger;
    private ScriptExecutionError error;
    @JsonFormat(shape = JsonFormat.Shape.STRING, pattern = "yyyy-MM-dd'T'HH:mm:ss.SSS")
    private LocalDateTime startTime;
    private Long executionTime;
    @Builder.Default
    private boolean cancelled = false;

    public void updateExecutionTime() {
        executionTime = getStartTime().until(LocalDateTime.now(), ChronoUnit.MILLIS);
    }
}
