package org.correomqtt.business.scripting;

import lombok.*;
import org.correomqtt.business.model.ScriptExecutionDTO;
import org.graalvm.polyglot.Context;
import org.graalvm.polyglot.Value;

import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Future;

@NoArgsConstructor
@AllArgsConstructor
@Getter
@Setter
@Builder
public class ExecutionDTO {

    private ExecutorService executorService;
    private Context context;
    private StringBuilder log;
    private ScriptExecutionDTO scriptExecutionDTO;
    private long startTime;

}
