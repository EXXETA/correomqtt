package org.correomqtt.business.scripting;

import lombok.*;
import org.graalvm.polyglot.Context;

import java.io.PipedOutputStream;
import java.util.concurrent.ExecutorService;

@NoArgsConstructor
@AllArgsConstructor
@Getter
@Setter
@Builder
class ExecutionContextDTO {

    private Context context;
    private ExecutionDTO executionDTO;

    private PipedOutputStream out;

}
